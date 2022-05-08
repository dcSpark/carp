use std::sync::{Arc, Mutex};

use crate::tasks::byron::byron_inputs::ByronInputTask;
use crate::tasks::byron::byron_outputs::ByronOutputTask;
use crate::tasks::byron::byron_txs::ByronTransactionTask;
use crate::tasks::database_task::DatabaseTask;
use crate::tasks::utils::TaskPerfAggregator;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::primitives::byron::{self};
use shred::{DispatcherBuilder, World};
use tokio::runtime::Handle;

pub async fn process_byron_block(
    txn: &DatabaseTransaction,
    block: (&byron::Block, &BlockModel),
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let handle = Handle::current();

    let mut world = World::empty();

    let byron_input_task = ByronInputTask::new(txn, block, &handle, perf_aggregator.clone());
    let byron_output_task = ByronOutputTask::new(txn, block, &handle, perf_aggregator.clone());
    let byron_transaction_task =
        ByronTransactionTask::new(txn, block, &handle, perf_aggregator.clone());

    let mut dispatcher = DispatcherBuilder::new()
        .with(
            byron_transaction_task,
            ByronTransactionTask::NAME,
            &ByronTransactionTask::DEPENDENCIES,
        )
        .with(
            byron_output_task,
            ByronOutputTask::NAME,
            &ByronOutputTask::DEPENDENCIES,
        )
        .with(
            byron_input_task,
            ByronInputTask::NAME,
            &ByronInputTask::DEPENDENCIES,
        )
        .build();
    dispatcher.setup(&mut world);
    dispatcher.dispatch(&world);

    Ok(())
}
