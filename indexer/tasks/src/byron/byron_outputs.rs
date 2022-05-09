extern crate shred;
use std::sync::{Arc, Mutex};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::{
    codec::utils::MaybeIndefArray,
    ledger::primitives::{
        byron::{self, TxOut},
        Fragment,
    },
};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::collections::BTreeMap;

use crate::{
    database_task::{
        BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta, TaskBuilder, TaskRegistryEntry,
    },
    era_common::{get_truncated_address, AddressInBlock},
    utils::TaskPerfAggregator,
};

use super::{byron_address::ByronAddressTask, byron_txs::ByronTransactionTask};

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: Read<'a, Vec<TransactionModel>>,
    byron_addresses: Read<'a, BTreeMap<Vec<u8>, AddressInBlock>>,
    byron_outputs: Write<'a, Vec<TransactionOutputModel>>,
}

pub struct ByronOutputTask<'a> {
    db_tx: &'a DatabaseTransaction,
    block: BlockInfo<'a, byron::Block>,
    handle: &'a tokio::runtime::Handle,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, byron::Block> for ByronOutputTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(ByronOutputTask);
    const DEPENDENCIES: &'static [&'static str] = &[
        name_of_type!(ByronTransactionTask),
        name_of_type!(ByronAddressTask),
    ];

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
    fn get_name(&self) -> &'static str {
        ByronOutputTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        ByronOutputTask::DEPENDENCIES
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
        let task = ByronOutputTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Byron(ByronTaskRegistryEntry { builder: &ByronOutputTaskBuilder })
}

impl<'a> System<'a> for ByronOutputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_outputs(
                self.db_tx,
                self.block,
                bundle.byron_txs.as_slice(),
                &bundle.byron_addresses,
            ))
            .unwrap();
        *bundle.byron_outputs = result;

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
    byron_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            let tx_outputs: Vec<_> = main_block
                .body
                .tx_payload
                .iter()
                .map(|payload| &payload.transaction.outputs)
                .zip(byron_txs)
                .collect();

            if tx_outputs.is_empty() {
                return Ok(vec![]);
            }

            // note: outputs have to be added before inputs
            Ok(insert_byron_outputs(db_tx, byron_addresses, &tx_outputs).await?)
        }
    }
}

async fn insert_byron_outputs(
    txn: &DatabaseTransaction,
    address_map: &BTreeMap<Vec<u8>, AddressInBlock>,
    outputs: &[(&MaybeIndefArray<TxOut>, &TransactionModel)],
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let result = TransactionOutput::insert_many(
        outputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(
                |((output_index, output), tx_id)| TransactionOutputActiveModel {
                    payload: Set(output.encode_fragment().unwrap()),
                    address_id: Set(address_map
                        .get(get_truncated_address(
                            &output.address.encode_fragment().unwrap(),
                        ))
                        .unwrap()
                        .model
                        .id),
                    tx_id: Set(tx_id.id),
                    output_index: Set(output_index as i32),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(txn)
    .await?;

    Ok(result)
}
