#![allow(non_snake_case)]
use crate::genesis_helpers::{
    addr_as_byron, db_output_as_byron_and_coin, in_memory_db_conn, new_perf_aggregator,
    pubkey_as_byron, some_block, OwnedBlockInfo, GENESIS_HASH,
};
use entity::{
    address, block,
    block::EraValue,
    sea_orm::EntityTrait,
    sea_orm::{
        ConnectionTrait, DatabaseBackend, DatabaseTransaction, DbConn, DbErr, Schema,
        TransactionTrait,
    },
    transaction, transaction_output,
};
use genesis_helpers::GenesisBlockBuilder;
use std::sync::{Arc, Mutex};
use tasks::{
    execution_plan::ExecutionPlan, genesis::genesis_executor::process_genesis_block,
    utils::TaskPerfAggregator,
};

mod genesis_helpers;

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
async fn process_genesis_block__when_genesis_tx_task_then_txs_in_correct_order() {
    // Given
    let conn = in_memory_db_conn().await;
    setup_schema(&conn).await;

    let block_info = some_block();

    let protocol_magic = block_info.1.protocol_magic;
    let mut avvm_in_block: Vec<_> = block_info
        .1
        .avvm_distr
        .clone()
        .into_iter()
        .map(|(pubkey, coin)| (pubkey_as_byron(&pubkey, protocol_magic), coin))
        .collect();

    let non_avvm_in_block: Vec<_> = block_info
        .1
        .non_avvm_balances
        .clone()
        .into_iter()
        .map(|(addr, coin)| (addr_as_byron(addr), coin))
        .collect();

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
    let _txs = transaction::Entity::find().all(&conn).await.unwrap();
    let outputs = transaction_output::Entity::find().all(&conn).await.unwrap();
    let _addresses = address::Entity::find().all(&conn).await.unwrap();

    // Transactions kept order
    // Outputs kept order
    let distr_and_balances_in_db: Vec<_> =
        outputs.iter().map(db_output_as_byron_and_coin).collect();
    let distr_and_balances_in_block = {
        avvm_in_block.extend(non_avvm_in_block);
        avvm_in_block
    };
    assert_eq!(distr_and_balances_in_block, distr_and_balances_in_db);
    // Addresses paired with correct first transaction
}
