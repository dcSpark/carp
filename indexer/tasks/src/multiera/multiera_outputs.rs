extern crate shred;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self, TransactionBody, TransactionBodyComponent};
use pallas::ledger::primitives::Fragment;
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    era_common::{get_truncated_address, AddressInBlock},
    utils::TaskPerfAggregator,
};

use super::multiera_address::MultieraAddressTask;

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    multiera_addresses: Read<'a, BTreeMap<Vec<u8>, AddressInBlock>>,
    multiera_outputs: Write<'a, Vec<TransactionOutputModel>>,
}

pub struct MultieraOutputTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraOutputTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraOutputTask);
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(MultieraAddressTask)];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, alonzo::Block>,
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

struct MultieraOutputTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraOutputTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraOutputTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraOutputTask::DEPENDENCIES
    }

    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, alonzo::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        _properties: &ini::Properties,
    ) {
        let task = MultieraOutputTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraOutputTaskBuilder })
}

impl<'a> System<'a> for MultieraOutputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_output(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &bundle.multiera_addresses,
            ))
            .unwrap();
        *bundle.multiera_outputs = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

struct QueuedOutput {
    // note: no need to use a map type
    // because the pair <tx_id, idx> should only ever be inserted once
    tx_id: i64,
    idx: usize,
    payload: Vec<u8>,
    address: Vec<u8>, // pallas::crypto::hash::Hash<32>
}

async fn handle_output(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let mut queued_output = Vec::<QueuedOutput>::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            #[allow(clippy::single_match)]
            match component {
                TransactionBodyComponent::Outputs(outputs) => {
                    for (idx, output) in outputs.iter().enumerate() {
                        queue_output(
                            &mut queued_output,
                            tx_body,
                            cardano_transaction.id,
                            output,
                            idx,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    Ok(insert_outputs(addresses, &queued_output, db_tx).await?)
}

fn queue_output(
    queued_output: &mut Vec<QueuedOutput>,
    tx_body: &TransactionBody,
    tx_id: i64,
    output: &alonzo::TransactionOutput,
    idx: usize,
) {
    use cardano_multiplatform_lib::address::Address;
    let addr = Address::from_bytes(output.address.to_vec())
        .map_err(|e| panic!("{:?}{:?}", e, tx_body.to_hash().to_vec()))
        .unwrap();

    queued_output.push(QueuedOutput {
        payload: output.encode_fragment().unwrap(),
        address: addr.to_bytes(),
        tx_id,
        idx,
    });
}

async fn insert_outputs(
    address_to_model_map: &BTreeMap<Vec<u8>, AddressInBlock>,
    queued_output: &[QueuedOutput],
    txn: &DatabaseTransaction,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    if queued_output.is_empty() {
        return Ok(vec![]);
    };

    Ok(
        TransactionOutput::insert_many(queued_output.iter().map(|entry| {
            TransactionOutputActiveModel {
                address_id: Set(address_to_model_map
                    .get(get_truncated_address(&entry.address))
                    .unwrap()
                    .model
                    .id),
                tx_id: Set(entry.tx_id),
                payload: Set(entry.payload.clone()),
                output_index: Set(entry.idx as i32),
                ..Default::default()
            }
        }))
        .exec_many_with_returning(txn)
        .await?,
    )
}
