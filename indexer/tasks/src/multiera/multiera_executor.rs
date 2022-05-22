use std::sync::{Arc, Mutex};

use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockInfo;
use crate::dsl::database_task::TaskRegistryEntry;
use crate::execution_plan::ExecutionPlan;
use crate::utils::find_task_registry_entry;
use crate::utils::TaskPerfAggregator;
use entity::sea_orm::{prelude::*, DatabaseTransaction};
use pallas::ledger::primitives::alonzo;
use shred::{DispatcherBuilder, World};
use tokio::runtime::Handle;

pub async fn process_multiera_block(
    txn: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    exec_plan: &ExecutionPlan,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let ep_start_time = std::time::Instant::now();

    let handle = Handle::current();

    let mut world = World::empty();

    let mut dispatcher_builder = DispatcherBuilder::new();

    for (task_name, val) in exec_plan.0.iter() {
        if let toml::value::Value::Table(_task_props) = val {
            let entry = find_task_registry_entry(task_name);
            match &entry {
                None => {
                    panic!("Could not find task named {}", task_name);
                }
                Some(task) => {
                    if let TaskRegistryEntry::Multiera(entry) = task {
                        entry.builder.maybe_add_task(
                            &mut dispatcher_builder,
                            txn,
                            block,
                            &handle,
                            perf_aggregator.clone(),
                            val,
                        );
                    }
                }
            }
        }
    }
    if !dispatcher_builder.is_empty() {
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
