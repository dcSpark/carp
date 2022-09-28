use anyhow::anyhow;
use entity::block::EraValue;
use std::fs;
use std::sync::{Arc, Mutex};
use tasks::dsl::database_task::BlockGlobalInfo;

use cardano_multiplatform_lib::genesis::byron::{config::GenesisData, parse::parse};
use entity::sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use migration::DbErr;
use tasks::utils::TaskPerfAggregator;
use tasks::{execution_plan::ExecutionPlan, genesis::genesis_executor::process_genesis_block};

const GENESIS_MAINNET: &str = "./genesis/mainnet-byron-genesis.json";
const GENESIS_PREVIEW: &str = "./genesis/preview-byron-genesis.json";
const GENESIS_PREPROD: &str = "./genesis/preprod-byron-genesis.json";
const GENESIS_TESTNET: &str = "./genesis/testnet-byron-genesis.json";

pub async fn process_genesis(
    conn: &DatabaseConnection,
    network: &str,
    exec_plan: Arc<ExecutionPlan>,
) -> anyhow::Result<()> {
    // https://github.com/txpipe/oura/blob/67b01e8739ed2927ced270e08daea74b03bcc7f7/src/sources/common.rs#L91
    let genesis_path = match network {
        "mainnet" => GENESIS_MAINNET,
        "testnet" => GENESIS_TESTNET,
        "preview" => GENESIS_PREVIEW,
        "preprod" => GENESIS_PREPROD,
        rest => {
            return Err(anyhow!(
                "{} is invalid. NETWORK must be either mainnet/preview/preprod/testnet",
                rest
            ))
        }
    };

    let task_perf_aggregator = Arc::new(Mutex::new(TaskPerfAggregator::default()));

    tracing::info!("Parsing genesis file...");
    let mut time_counter = std::time::Instant::now();

    let file = fs::File::open(genesis_path).expect("Failed to open genesis file");
    let genesis_file: Box<GenesisData> = Box::new(parse(file));

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
    let genesis_hash = genesis_file.genesis_prev.to_bytes();
    tracing::info!(
        "Starting sync based on genesis hash {}",
        hex::encode(genesis_hash.clone())
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
