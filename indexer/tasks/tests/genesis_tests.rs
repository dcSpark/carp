use crate::genesis_helpers::{OwnedBlockInfo, GENESIS_HASH};
use cardano_multiplatform_lib::address::ByronAddress;
use cardano_multiplatform_lib::chain_crypto::{Ed25519, KeyPair, PublicKey};
use cardano_multiplatform_lib::from_bytes;
use cardano_multiplatform_lib::utils::BigNum;
use entity::prelude::{AddressModel, TransactionModel, TransactionOutputModel};
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
use rand::{CryptoRng, Rng, RngCore, SeedableRng};
use std::collections::BTreeMap;
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

fn new_pubkey<R: RngCore + CryptoRng>(rng: &mut R) -> PublicKey<Ed25519> {
    let (_, pubkey) = KeyPair::<Ed25519>::generate(rng).into_keys();
    pubkey
}

#[tokio::test]
async fn process_genesis_block__when_genesis_tx_task_then_txs_in_correct_order() {
    // Given
    let conn = in_memory_db_conn().await;
    setup_schema(&conn).await;
    let mut rng = new_rng();
    let pubkey1 = new_pubkey(&mut rng);
    let pubkey2 = new_pubkey(&mut rng);
    let pubkey3 = new_pubkey(&mut rng);
    let coin1 = BigNum::from(100);
    let coin2 = BigNum::from(200);
    let coin3 = BigNum::from(50);

    let block_info = GenesisBlockBuilder::default()
        .with_voucher(pubkey1, coin1)
        .with_voucher(pubkey2, coin2)
        .with_voucher(pubkey3, coin3)
        .build();

    let avvm_in_block = block_info.1.avvm_distr.clone();

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
    txs.iter().for_each(|tx| println!("Transaction: {:?}", tx));
    let outputs = transaction_output::Entity::find().all(&conn).await.unwrap();
    outputs.iter().for_each(|tx| println!("Output: {:?}", tx));
    let addresses = address::Entity::find().all(&conn).await.unwrap();
    addresses
        .iter()
        .for_each(|tx| println!("Address: {:?}", tx));

    let avvm_in_db = reconstruct_original_transactions(&outputs);

    assert_eq!(avvm_in_block, avvm_in_db);
}

fn reconstruct_original_transactions(
    outputs: &Vec<TransactionOutputModel>,
) -> BTreeMap<PublicKey<Ed25519>, BigNum> {
    // Get value and address from output
    let mut original_transactions = BTreeMap::new();

    for output in outputs {
        let payload = output.payload.clone();
        let cml_output = cardano_multiplatform_lib::TransactionOutput::from_bytes(payload).unwrap();
        let coin = cml_output.amount().coin();
        let address = cml_output.address();
        let byron_address = ByronAddress::from_address(&address).unwrap();
        let pubkey = todo!("fun conversion stuff that looks really easy");
        original_transactions.insert(pubkey, coin);
    }

    original_transactions
}
