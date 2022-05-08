extern crate shred;
use std::sync::{Arc, Mutex};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::{
    byron::{self},
    Fragment,
};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::collections::BTreeMap;

use crate::tasks::{
    database_task::{
        BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta, TaskBuilder, TaskRegistryEntry,
    },
    era_common::AddressInBlock,
    utils::TaskPerfAggregator,
};

use super::byron_txs::ByronTransactionTask;

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: Read<'a, Vec<TransactionModel>>,
    byron_addresses: Write<'a, BTreeMap<Vec<u8>, AddressInBlock>>,
}

pub struct ByronAddressTask<'a> {
    db_tx: &'a DatabaseTransaction,
    block: BlockInfo<'a, byron::Block>,
    handle: &'a tokio::runtime::Handle,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, byron::Block> for ByronAddressTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(ByronAddressTask);
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(ByronTransactionTask)];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, byron::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Self {
        Self {
            db_tx,
            block,
            handle,
            perf_aggregator,
        }
    }
}

struct ByronOutputTaskBuilder;
impl<'a> TaskBuilder<'a, byron::Block> for ByronOutputTaskBuilder {
    fn get_name() -> &'static str {
        ByronAddressTask::TASK_NAME
    }
    fn get_dependencies() -> &'static [&'static str] {
        ByronAddressTask::DEPENDENCIES
    }
    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, byron::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        _properties: &ini::Properties,
    ) {
        let task = ByronAddressTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, Self::get_name(), Self::get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Byron(ByronTaskRegistryEntry {name: ByronAddressTask::TASK_NAME, builder: &ByronOutputTaskBuilder })
}

impl<'a> System<'a> for ByronAddressTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_outputs(
                self.db_tx,
                self.block,
                bundle.byron_txs.as_slice(),
            ))
            .unwrap();
        *bundle.byron_addresses = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_outputs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
    byron_txs: &[TransactionModel],
) -> Result<BTreeMap<Vec<u8>, AddressInBlock>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(BTreeMap::<Vec<u8>, AddressInBlock>::default()),
        byron::Block::MainBlock(main_block) => {
            let tx_outputs: Vec<_> = main_block
                .body
                .tx_payload
                .iter()
                .map(|payload| &payload.transaction.outputs)
                .zip(byron_txs)
                .collect();

            if tx_outputs.is_empty() {
                return Ok(BTreeMap::<Vec<u8>, AddressInBlock>::default());
            }
            // insert addresses
            let address_map = crate::tasks::era_common::insert_addresses(
                &tx_outputs
                    .iter()
                    .flat_map(|pair| pair.0.iter())
                    .map(|output| output.address.encode_fragment().unwrap())
                    .collect(),
                db_tx,
            )
            .await?;

            Ok(address_map)
        }
    }
}
