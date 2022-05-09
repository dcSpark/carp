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
use pallas::ledger::primitives::Fragment;
use pallas::{
    codec::utils::KeyValuePairs,
    ledger::primitives::alonzo::{self, AuxiliaryData, Metadatum, MetadatumLabel},
};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    utils::TaskPerfAggregator,
};

use super::multiera_txs::MultieraTransactionTask;

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    multiera_metadata: Write<'a, Vec<TransactionMetadataModel>>,
}

pub struct MultieraMetadataTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraMetadataTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraMetadataTask);
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(MultieraTransactionTask)];

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

struct MultieraMetadataTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraMetadataTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraMetadataTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraMetadataTask::DEPENDENCIES
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
        let task = MultieraMetadataTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraMetadataTaskBuilder })
}

impl<'a> System<'a> for MultieraMetadataTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_metadata(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
            ))
            .unwrap();
        *bundle.multiera_metadata = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_metadata(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
) -> Result<Vec<TransactionMetadataModel>, DbErr> {
    let mut metadata_map =
        BTreeMap::<i64 /* id */, &KeyValuePairs<MetadatumLabel, Metadatum>>::default();
    for (idx, data) in block.1.auxiliary_data_set.iter() {
        let tx_id = &multiera_txs[*idx as usize].id;
        let opt_entries = match data {
            AuxiliaryData::Shelley(metadata) => Some(metadata),
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                ..
            } => Some(transaction_metadata),
            AuxiliaryData::Alonzo(data) => data.metadata.as_ref(),
        };
        opt_entries.and_then(|entries| metadata_map.insert(*tx_id, entries));
    }

    if metadata_map.is_empty() {
        return Ok(vec![]);
    };

    Ok(TransactionMetadata::insert_many(
        metadata_map
            .iter()
            .flat_map(|(tx_id, metadata)| metadata.iter().zip(std::iter::repeat(tx_id)))
            .map(
                |((label, metadata), tx_id)| TransactionMetadataActiveModel {
                    tx_id: Set(*tx_id),
                    label: Set(u64::from(label).to_le_bytes().to_vec()),
                    payload: Set(metadata.encode_fragment().unwrap()),
                },
            ),
    )
    .exec_many_with_returning(db_tx)
    .await?)
}
