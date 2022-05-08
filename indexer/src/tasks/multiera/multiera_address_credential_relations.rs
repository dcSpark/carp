extern crate shred;

use std::{
    collections::{BTreeMap, BTreeSet},
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
    tasks::{
        database_task::{
            BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
        },
        era_common::AddressInBlock,
        utils::TaskPerfAggregator,
    },
    types::AddressCredentialRelationValue,
};

use super::{
    multiera_address::MultieraAddressTask, multiera_stake_credentials::MultieraStakeCredentialTask,
};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_queued_addresses_relations: Read<'a, BTreeSet<QueuedAddressCredentialRelation>>,
    multiera_stake_credential: Read<'a, BTreeMap<Vec<u8>, StakeCredentialModel>>,
    multiera_addresses: Read<'a, BTreeMap<Vec<u8>, AddressInBlock>>,
}

pub struct MultieraAddressCredentialRelation<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraAddressCredentialRelation<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraAddressCredentialRelation);
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

struct MultieraAddressCredentialRelationTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraAddressCredentialRelationTaskBuilder {
    fn get_name() -> &'static str {
        MultieraAddressCredentialRelation::TASK_NAME
    }
    fn get_dependencies() -> &'static [&'static str] {
        MultieraAddressCredentialRelation::DEPENDENCIES
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
        let task = MultieraAddressCredentialRelation::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, Self::get_name(), Self::get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry {name: MultieraAddressCredentialRelation::TASK_NAME, builder: &MultieraAddressCredentialRelationTaskBuilder })
}

impl<'a> System<'a> for MultieraAddressCredentialRelation<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        self.handle
            .block_on(handle_address_credential_relation(
                self.db_tx,
                &bundle.multiera_stake_credential,
                &bundle.multiera_addresses,
                &bundle.multiera_queued_addresses_relations,
            ))
            .unwrap();

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct QueuedAddressCredentialRelation {
    pub address: Vec<u8>,
    pub stake_credential: Vec<u8>, // pallas::crypto::hash::Hash<32>
    pub address_relation: AddressCredentialRelationValue,
}

async fn handle_address_credential_relation(
    db_tx: &DatabaseTransaction,
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
    queued_address_credential: &BTreeSet<QueuedAddressCredentialRelation>,
) -> Result<Vec<AddressCredentialModel>, DbErr> {
    if queued_address_credential.is_empty() {
        return Ok(vec![]);
    }

    let mut new_address_map = BTreeMap::<&Vec<u8>, &AddressModel>::default();
    multiera_addresses.values().for_each(|next| {
        if next.is_new {
            new_address_map.insert(&next.model.payload, &next.model);
        }
    });

    let mut to_add: Vec<AddressCredentialActiveModel> = vec![];
    for entry in queued_address_credential {
        // we can ignore addresses we've already seen before
        if let Some(&address_model) = new_address_map.get(&entry.address) {
            to_add.push(AddressCredentialActiveModel {
                credential_id: Set(multiera_stake_credential
                    .get(&entry.stake_credential)
                    .unwrap()
                    .id),
                address_id: Set(address_model.id),
                relation: Set(entry.address_relation as i32),
            });
        }
    }

    match to_add.is_empty() {
        true => Ok(vec![]),
        false => Ok(AddressCredential::insert_many(to_add.clone())
            .exec_many_with_returning(db_tx)
            .await?),
    }
}
