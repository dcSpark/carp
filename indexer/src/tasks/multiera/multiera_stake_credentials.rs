extern crate shred;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    relation_map::RelationMap,
    tasks::{
        database_task::{
            BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
        },
        utils::TaskPerfAggregator,
    },
};

use super::{
    multiera_unused_input::MultieraUnusedInputTask, multiera_used_inputs::MultieraUsedInputTask,
};

#[derive(SystemData)]
pub struct Data<'a> {
    vkey_relation_map: Read<'a, RelationMap>,
    multiera_stake_credential: Write<'a, BTreeMap<Vec<u8>, StakeCredentialModel>>,
}

pub struct MultieraStakeCredentialTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraStakeCredentialTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraStakeCredentialTask);
    // note: has to be done after inputs as they may add new creds
    const DEPENDENCIES: &'static [&'static str] = &[
        name_of_type!(MultieraUsedInputTask),
        name_of_type!(MultieraUnusedInputTask),
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

struct MultieraStakeCredentialTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraStakeCredentialTaskBuilder {
    fn get_name() -> &'static str {
        MultieraStakeCredentialTask::TASK_NAME
    }
    fn get_dependencies() -> &'static [&'static str] {
        MultieraStakeCredentialTask::DEPENDENCIES
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
        let task = MultieraStakeCredentialTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, Self::get_name(), Self::get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry {name: MultieraStakeCredentialTask::TASK_NAME, builder: &MultieraStakeCredentialTaskBuilder })
}

impl<'a> System<'a> for MultieraStakeCredentialTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_stake_credentials(
                self.db_tx,
                &bundle.vkey_relation_map,
            ))
            .unwrap();
        *bundle.multiera_stake_credential = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_stake_credentials(
    db_tx: &DatabaseTransaction,
    vkey_relation_map: &RelationMap,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    let cred_to_model_map = insert_stake_credentials(
        &vkey_relation_map
            .0
            .values()
            .flat_map(|cred_to_model| cred_to_model.keys())
            .map(|pallas| pallas.to_vec())
            .collect(),
        db_tx,
    )
    .await?;
    Ok(cred_to_model_map)
}

async fn insert_stake_credentials(
    deduplicated_creds: &BTreeSet<Vec<u8>>,
    txn: &DatabaseTransaction,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    let mut result_map = BTreeMap::<Vec<u8>, StakeCredentialModel>::default();

    if deduplicated_creds.is_empty() {
        return Ok(result_map);
    }

    // 1) Add credentials that were already in the DB
    {
        let mut found_credentials = StakeCredential::find()
            .filter(
                Condition::any()
                    .add(StakeCredentialColumn::Credential.is_in(deduplicated_creds.clone())),
            )
            .all(txn)
            .await?;

        result_map.extend(
            found_credentials
                .drain(..)
                .map(|model| (model.credential.clone(), model)),
        );
    }

    // 2) Add credentials that weren't in the DB
    {
        // check which credentials weren't found in the DB and prepare to add them
        let credentials_to_add: Vec<StakeCredentialActiveModel> = deduplicated_creds
            .iter()
            .filter(|&credential| !result_map.contains_key(credential))
            .map(|credential| StakeCredentialActiveModel {
                credential: Set(credential.to_vec()),
                ..Default::default()
            })
            .collect();

        // add the new entires into the DB, then add them to our result mapping
        if !credentials_to_add.is_empty() {
            let mut additions = StakeCredential::insert_many(credentials_to_add)
                .exec_many_with_returning(txn)
                .await?;
            additions.drain(..).for_each(|model| {
                result_map.insert(model.credential.clone(), model);
            });
        }
    }

    Ok(result_map)
}
