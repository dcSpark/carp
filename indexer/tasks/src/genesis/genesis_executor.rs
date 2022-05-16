use std::sync::{Arc, Mutex};

use crate::database_task::BlockInfo;
use crate::database_task::TaskRegistryEntry;
use crate::execution_plan::ExecutionPlan;
use crate::utils::find_task_registry_entry;
use crate::utils::TaskPerfAggregator;
use cardano_multiplatform_lib::genesis::byron::config::GenesisData;
use entity::sea_orm::{prelude::*, DatabaseTransaction};
use shred::{DispatcherBuilder, World};
use tokio::runtime::Handle;

pub async fn process_genesis_block(
    txn: &DatabaseTransaction,
    block: BlockInfo<'_, GenesisData>,
    exec_plan: &ExecutionPlan,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let ep_start_time = std::time::Instant::now();

    let handle = Handle::current();

    let mut world = World::empty();

    let mut dispatcher_builder = DispatcherBuilder::new();

    let mut has_task = false;
    for task_name in exec_plan.0.sections().flatten() {
        let entry = find_task_registry_entry(task_name);
        match &entry {
            None => {
                panic!("Could not find task named {}", task_name);
            }
            Some(task) => {
                if let TaskRegistryEntry::Genesis(entry) = task {
                    has_task = has_task
                        || entry.builder.maybe_add_task(
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

    if has_task {
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);
        dispatcher.dispatch(&world);
    }

    perf_aggregator
        .lock()
        .unwrap()
        .add_to_total(&ep_start_time.elapsed());

    Ok(())
}
