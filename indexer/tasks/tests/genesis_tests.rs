use crate::genesis_helpers::{OwnedBlockInfo, GENESIS_HASH};
use cardano_multiplatform_lib::chain_crypto::{Ed25519, KeyPair};
use cardano_multiplatform_lib::utils::BigNum;
use entity::sea_orm::DatabaseConnection;
use entity::{
    address, block,
    block::EraValue,
    sea_orm::EntityTrait,
    sea_orm::{
        ConnectionTrait, Database, DatabaseBackend, DatabaseTransaction, DbConn, DbErr, Schema,
        TransactionTrait,
    },
    transaction, transaction_output,
};
use genesis_helpers::GenesisBlockBuilder;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::{Arc, Mutex};
use tasks::dsl::task_macro::alonzo::Coin;
use tasks::execution_plan::ExecutionPlan;
use tasks::genesis::genesis_executor::process_genesis_block;
use tasks::utils::TaskPerfAggregator;

mod genesis_helpers;

async fn in_memory_db_conn() -> DbConn {
    Database::connect("sqlite::memory:").await.unwrap()
}

fn new_perf_aggregator() -> Arc<Mutex<TaskPerfAggregator>> {
    Default::default()
}

async fn wrap_process_genesis_block(
    txn: &DatabaseTransaction,
    owned_block_info: OwnedBlockInfo,
    exec_plan: Arc<ExecutionPlan>,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let block_info = (
        owned_block_info.0.as_str(),
        &owned_block_info.1,
        &owned_block_info.2,
    );
    process_genesis_block(txn, block_info, &exec_plan, perf_aggregator.clone())
        .await
        .unwrap();
    Ok(())
}

async fn setup_schema(db: &DbConn) {
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt_for_blocks = schema.create_table_from_entity(block::Entity);
    let stmt_for_txs = schema.create_table_from_entity(transaction::Entity);
    let stmt_for_addresses = schema.create_table_from_entity(address::Entity);
    let stmt_for_outputs = schema.create_table_from_entity(transaction_output::Entity);

    let builder = db.get_database_backend();

    db.execute(builder.build(&stmt_for_blocks)).await.unwrap();
    db.execute(builder.build(&stmt_for_txs)).await.unwrap();
    db.execute(builder.build(&stmt_for_outputs)).await.unwrap();
    db.execute(builder.build(&stmt_for_addresses))
        .await
        .unwrap();
}

const RNG_SEED: [u8; 32] = [6; 32];

fn new_rng() -> StdRng {
    StdRng::from_seed(RNG_SEED)
}

#[tokio::test]
async fn process_genesis_block__when_genesis_block_task_then_added_to_db() {
    // Given
    let conn = in_memory_db_conn().await;
    setup_schema(&conn).await;
    let block_info = GenesisBlockBuilder::default().build();
    let exec_plan = Arc::new(ExecutionPlan::from(vec!["GenesisBlockTask"]));
    let perf_aggregator = new_perf_aggregator();

    let expected = block::Model {
        id: 1,
        era: EraValue::Byron.into(),
        hash: GENESIS_HASH.into(),
        height: 0,
        epoch: 0,
        slot: 0,
    };

    // When
    conn.transaction(|db_tx| {
        Box::pin(wrap_process_genesis_block(
            db_tx,
            block_info,
            exec_plan.clone(),
            perf_aggregator.clone(),
        ))
    })
    .await
    .unwrap();

    // Then
    let actual = block::Entity::find().one(&conn).await.unwrap().unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn process_genesis_block__when_genesis_tx_test_then_includes_txs() {
    // Given
    let conn = in_memory_db_conn().await;
    setup_schema(&conn).await;
    let mut rng = new_rng();
    let (_, pubkey) = KeyPair::<Ed25519>::generate(&mut rng).into_keys();
    let coin = BigNum::from(100);
    let block_info = GenesisBlockBuilder::default()
        .with_voucher(pubkey, coin)
        .build();
    let exec_plan = Arc::new(ExecutionPlan::from(vec![
        "GenesisBlockTask",
        "GenesisTransactionTask",
    ]));
    let perf_aggregator = new_perf_aggregator();

    // When
    conn.transaction(|db_tx| {
        Box::pin(wrap_process_genesis_block(
            db_tx,
            block_info,
            exec_plan.clone(),
            perf_aggregator.clone(),
        ))
    })
    .await
    .unwrap();

    // Then
    let txs = transaction::Entity::find().all(&conn).await.unwrap();
    dbg!(txs);
    let outputs = transaction_output::Entity::find().all(&conn).await.unwrap();
    dbg!(outputs);
    let addresses = address::Entity::find().all(&conn).await.unwrap();
    dbg!(addresses);
}
