use std::sync::{Arc, Mutex};

use crate::tasks::database_task::TaskRegistryEntry;
use crate::tasks::execution_plan::ExecutionPlan;
use crate::tasks::utils::find_task_registry_entry;
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
    exec_plan: &ExecutionPlan,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let handle = Handle::current();

    let mut world = World::empty();

    let mut dispatcher_builder = DispatcherBuilder::new();

    for task_name in exec_plan.0.sections().flatten() {
        let entry = find_task_registry_entry(task_name);
        match &entry {
            None => {
                panic!("Could not find task named {}", task_name);
            }
            Some(task) => {
                if let TaskRegistryEntry::Byron(entry) = task {
                    entry.builder.add_task(
                        &mut dispatcher_builder,
                        txn,
                        block,
                        &handle,
                        perf_aggregator.clone(),
                        exec_plan.0.section(Some(task_name)).unwrap(),
                    );
                }
            }
        }
    }
    let mut dispatcher = dispatcher_builder.build();
    dispatcher.setup(&mut world);
    dispatcher.dispatch(&world);

    Ok(())
}
