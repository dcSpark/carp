use crate::tasks::utils::TaskPerfAggregator;
use entity::{prelude::*, sea_orm::DatabaseTransaction};
use pallas::ledger::primitives::byron;
use shred::DispatcherBuilder;
use std::sync::{Arc, Mutex};

pub trait DatabaseTaskMeta<'a, BlockType> {
    const TASK_NAME: &'static str;
    const DEPENDENCIES: &'static [&'static str];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: (&'a BlockType, &'a BlockModel),
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Self;
}

pub trait TaskBuilder<'a, BlockType> {
    fn get_name() -> &'static str
    where
        Self: Sized;
    fn get_dependencies() -> &'static [&'static str]
    where
        Self: Sized;

    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: (&'a BlockType, &'a BlockModel),
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    );
}

#[derive(Copy, Clone)]
pub enum TaskRegistryEntry {
    Byron(ByronTaskRegistryEntry),
}
#[derive(Copy, Clone)]
pub struct ByronTaskRegistryEntry {
    pub name: &'static str,
    pub builder: &'static (dyn for<'a> TaskBuilder<'a, byron::Block> + Sync),
}

inventory::collect!(TaskRegistryEntry);
