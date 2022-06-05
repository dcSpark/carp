use crate::genesis_helpers::{OwnedBlockInfo, GENESIS_HASH};
use entity::{
    block,
    block::EraValue,
    sea_orm::EntityTrait,
    sea_orm::{
        ConnectionTrait, Database, DatabaseBackend, DatabaseTransaction, DbConn, DbErr, Schema,
        TransactionTrait,
    },
};
use genesis_helpers::GenesisBlockBuilder;
use std::sync::{Arc, Mutex};
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

#[tokio::test]
async fn process_genesis_block__empty_exec_plan_ok() {
    let conn = in_memory_db_conn().await;
    let block_info = GenesisBlockBuilder::default().build();
    let exec_plan = Arc::new(ExecutionPlan(Default::default()));
    let perf_aggregator = new_perf_aggregator();

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
}

async fn setup_schema(db: &DbConn) {
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(block::Entity);

    let builder = db.get_database_backend();
    db.execute(builder.build(&stmt)).await.unwrap();
}

#[tokio::test]
async fn process_genesis_block__when_genesis_block_task_then_added_to_db() {
    let expected = block::Model {
        id: 1,
        era: EraValue::Byron.into(),
        hash: GENESIS_HASH.into(),
        height: 0,
        epoch: 0,
        slot: 0,
    };

    let conn = in_memory_db_conn().await;
    setup_schema(&conn).await;
    let block_info = GenesisBlockBuilder::default().build();
    let exec_plan = Arc::new(ExecutionPlan::from(vec!["GenesisBlockTask"]));
    let perf_aggregator = new_perf_aggregator();

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

    let actual = block::Entity::find().one(&conn).await.unwrap().unwrap();

    assert_eq!(expected, actual);
}
