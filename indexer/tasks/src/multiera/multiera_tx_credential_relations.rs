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
use pallas::ledger::primitives::alonzo::{self};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    utils::TaskPerfAggregator,
};

use super::{
    multiera_address::MultieraAddressTask, multiera_stake_credentials::MultieraStakeCredentialTask,
    relation_map::RelationMap,
};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_stake_credential: Read<'a, BTreeMap<Vec<u8>, StakeCredentialModel>>,
    vkey_relation_map: Read<'a, RelationMap>,
}

pub struct MultieraTxCredentialRelationTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraTxCredentialRelationTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraTxCredentialRelationTask);
    const DEPENDENCIES: &'static [&'static str] = &[
        name_of_type!(MultieraAddressTask),
        name_of_type!(MultieraStakeCredentialTask),
    ];

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

struct MultieraTxCredentialRelationTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraTxCredentialRelationTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraTxCredentialRelationTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraTxCredentialRelationTask::DEPENDENCIES
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
        let task = MultieraTxCredentialRelationTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraTxCredentialRelationTaskBuilder })
}

impl<'a> System<'a> for MultieraTxCredentialRelationTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        self.handle
            .block_on(handle_tx_credential_relations(
                self.db_tx,
                &bundle.multiera_stake_credential,
                &bundle.vkey_relation_map,
            ))
            .unwrap();

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_tx_credential_relations(
    db_tx: &DatabaseTransaction,
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    vkey_relation_map: &RelationMap,
) -> Result<(), DbErr> {
    let mut models: Vec<TxCredentialActiveModel> = vec![];
    for (tx_id, mapping) in vkey_relation_map.0.iter() {
        models.extend(mapping.iter().map(|(credential, relation)| {
            TxCredentialActiveModel {
                credential_id: Set(multiera_stake_credential
                    .get(&credential.to_vec())
                    .unwrap()
                    .id),
                tx_id: Set(*tx_id),
                relation: Set(*relation),
            }
        }));
    }

    // no entries to add if tx only had Byron addresses
    if !models.is_empty() {
        TxCredential::insert_many(models).exec(db_tx).await?;
    }
    Ok(())
}
