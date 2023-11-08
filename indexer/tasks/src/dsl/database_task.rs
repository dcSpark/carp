use crate::utils::TaskPerfAggregator;
use cml_chain::genesis::byron::config::GenesisData;
use entity::{block::EraValue, prelude::*, sea_orm::DatabaseTransaction};
use shred::DispatcherBuilder;
use std::sync::{Arc, Mutex};

/// Misc information about blocks that can't be computed from just the block data itself
pub struct BlockGlobalInfo {
    pub era: EraValue,
    pub epoch: Option<u64>,
    pub epoch_slot: Option<u64>,
}

pub type BlockInfo<'a, BlockType, BlockExtraType> = (
    &'a str, // hex-encoded cbor. Empty for genesis
    &'a BlockType,
    &'a BlockExtraType,
);

pub trait DatabaseTaskMeta<'a, BlockType, BlockExtraType, Configuration> {
    const TASK_NAME: &'static str;
    const DEPENDENCIES: &'static [&'static str];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, BlockType, BlockExtraType>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        config: &Configuration,
    ) -> Self;

    fn get_configuration(&self) -> &Configuration;

    fn should_add_task(
        block: BlockInfo<'a, BlockType, BlockExtraType>,
        properties: &toml::value::Value,
    ) -> bool;
}

pub trait TaskBuilder<'a, BlockType, BlockExtraType> {
    fn get_name(&self) -> &'static str;
    fn get_dependencies(&self) -> &'static [&'static str];

    fn maybe_add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, BlockType, BlockExtraType>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        properties: &toml::value::Value,
    ) -> bool;
}

#[derive(Copy, Clone)]
pub enum TaskRegistryEntry {
    Genesis(GenesisTaskRegistryEntry),
    Byron(ByronTaskRegistryEntry),
    Multiera(MultieraTaskRegistryEntry),
}

#[derive(Copy, Clone)]
pub struct GenesisTaskRegistryEntry {
    pub builder: &'static (dyn for<'a> TaskBuilder<'a, GenesisData, BlockGlobalInfo> + Sync),
}

#[derive(Copy, Clone)]
pub struct ByronTaskRegistryEntry {
    pub builder: &'static (dyn for<'a> TaskBuilder<'a, cml_multi_era::MultiEraBlock, BlockGlobalInfo> + Sync),
}

#[derive(Copy, Clone)]
pub struct MultieraTaskRegistryEntry {
    pub builder: &'static (dyn for<'a> TaskBuilder<'a, cml_multi_era::MultiEraBlock, BlockGlobalInfo> + Sync),
}

inventory::collect!(TaskRegistryEntry);
