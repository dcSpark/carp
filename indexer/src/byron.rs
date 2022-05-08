use std::sync::{Arc, Mutex};

use crate::tasks::byron::byron_outputs::ByronOutputTask;
use crate::tasks::database_task::DatabaseTaskMeta;
use crate::tasks::utils::find_byron_task_builder;
use crate::tasks::{
    byron::{byron_inputs::ByronInputTask, byron_txs::ByronTransactionTask},
    utils::TaskPerfAggregator,
};
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

    let tasks_to_run = vec![
        ByronTransactionTask::NAME,
        ByronOutputTask::NAME,
        ByronInputTask::NAME,
    ];

    let mut dispatcher_builder = DispatcherBuilder::new();
    for task in tasks_to_run {
        match find_byron_task_builder(task) {
            Some(builder) => {
                builder.builder.add_task(
                    &mut dispatcher_builder,
                    txn,
                    block,
                    &handle,
                    perf_aggregator.clone(),
                );
            }
            None => {
                panic!("Could not find task named {}", task);
            }
        }
    }
    let mut dispatcher = dispatcher_builder.build();
    dispatcher.setup(&mut world);
    dispatcher.dispatch(&world);

    Ok(())
}
