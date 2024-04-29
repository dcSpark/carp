use anyhow::anyhow;
use entity::block::EraValue;
use std::fs;
use std::sync::{Arc, Mutex};
use tasks::dsl::database_task::BlockGlobalInfo;

use cml_chain::genesis::byron::config::GenesisData;
use cml_chain::genesis::byron::parse::parse_genesis_data;

use cml_crypto::RawBytesEncoding;
use entity::sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use migration::DbErr;
use tasks::utils::TaskPerfAggregator;
use tasks::{execution_plan::ExecutionPlan, genesis::genesis_executor::process_genesis_block};

pub async fn process_genesis(
    conn: &DatabaseConnection,
    network: &str,
    genesis_folder: &str,
    exec_plan: Arc<ExecutionPlan>,
) -> anyhow::Result<()> {
    let task_perf_aggregator = Arc::new(Mutex::new(TaskPerfAggregator::default()));

    tracing::info!("Parsing genesis file...");
    let mut time_counter = std::time::Instant::now();

    let file = fs::File::open(format!("{}/{}-byron-genesis.json", genesis_folder, network))
        .expect("Failed to open genesis file");
    let genesis_file: Box<GenesisData> = Box::new(
        parse_genesis_data(file).map_err(|err| anyhow!("can't parse genesis data: {:?}", err))?,
    );

    tracing::info!(
        "Finished parsing genesis file after {:?}",
        time_counter.elapsed()
    );
    time_counter = std::time::Instant::now();

    tracing::info!("Inserting genesis data into database...");
    conn.transaction(|txn| {
        Box::pin(insert_genesis(
            txn,
            genesis_file,
            exec_plan.clone(),
            task_perf_aggregator.clone(),
        ))
    })
    .await?;

    tracing::info!(
        "Finished inserting genesis data after {:?}",
        time_counter.elapsed()
    );
    tracing::trace!(
        "Genesis task-wise time spent:\n{:#?}",
        task_perf_aggregator.lock().unwrap()
    );

    Ok(())
}

pub async fn insert_genesis(
    txn: &DatabaseTransaction,
    genesis_file: Box<GenesisData>,
    exec_plan: Arc<ExecutionPlan>,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let genesis_hash = genesis_file.genesis_prev.to_raw_bytes();
    tracing::info!(
        "Starting sync based on genesis hash {}",
        hex::encode(genesis_hash)
    );

    let block_global_info = BlockGlobalInfo {
        era: EraValue::Byron,
        epoch: None,
        epoch_slot: None,
    };

    process_genesis_block(
        txn,
        ("", &genesis_file, &block_global_info),
        &exec_plan,
        task_perf_aggregator.clone(),
    )
    .await?;

    Ok(())
}
