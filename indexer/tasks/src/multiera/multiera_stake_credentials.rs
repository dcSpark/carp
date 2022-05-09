extern crate shred;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use cardano_multiplatform_lib::{utils::ScriptHashNamespace, RequiredSignersSet};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    types::TxCredentialRelationValue,
    utils::TaskPerfAggregator,
};

use super::{
    multiera_unused_input::MultieraUnusedInputTask, multiera_used_inputs::MultieraUsedInputTask,
    relation_map::RelationMap,
};
use pallas::ledger::primitives::Fragment;

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    vkey_relation_map: Write<'a, RelationMap>,
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
    fn get_name(&self) -> &'static str {
        MultieraStakeCredentialTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
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
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraStakeCredentialTaskBuilder })
}

impl<'a> System<'a> for MultieraStakeCredentialTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_stake_credentials(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &mut bundle.vkey_relation_map,
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
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    for ((tx_body, cardano_transaction), witness_set) in block
        .1
        .transaction_bodies
        .iter()
        .zip(multiera_txs)
        .zip(block.1.transaction_witness_sets.iter())
    {
        queue_witness(
            vkey_relation_map,
            cardano_transaction.id,
            &cardano_multiplatform_lib::TransactionWitnessSet::from_bytes(
                witness_set.encode_fragment().unwrap(),
            )
            .unwrap(),
        );
        for component in tx_body.iter() {
            #[allow(clippy::single_match)]
            match component {
                TransactionBodyComponent::RequiredSigners(key_hashes) => {
                    for &signer in key_hashes.iter() {
                        let owner_credential =
                            pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(
                                signer,
                            )
                            .encode_fragment()
                            .unwrap();
                        vkey_relation_map.add_relation(
                            cardano_transaction.id,
                            &owner_credential.clone(),
                            TxCredentialRelationValue::RequiredSigner,
                        );
                    }
                }
                _ => {}
            }
        }
    }

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

fn queue_witness(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    witness_set: &cardano_multiplatform_lib::TransactionWitnessSet,
) {
    if let Some(vkeys) = witness_set.vkeys() {
        for i in 0..vkeys.len() {
            let vkey = vkeys.get(i);
            vkey_relation_map.add_relation(
                tx_id,
                RelationMap::keyhash_to_pallas(&vkey.vkey().public_key().hash()).as_slice(),
                TxCredentialRelationValue::Witness,
            );
        }
    }
    if let Some(scripts) = witness_set.native_scripts() {
        for i in 0..scripts.len() {
            let script = scripts.get(i);
            vkey_relation_map.add_relation(
                tx_id,
                RelationMap::scripthash_to_pallas(&script.hash(ScriptHashNamespace::NativeScript))
                    .as_slice(),
                TxCredentialRelationValue::Witness,
            );

            let vkeys_in_script = RequiredSignersSet::from(&script);
            for vkey_in_script in vkeys_in_script {
                vkey_relation_map.add_relation(
                    tx_id,
                    RelationMap::keyhash_to_pallas(&vkey_in_script).as_slice(),
                    TxCredentialRelationValue::InNativeScript,
                );
            }
        }
    }

    if let Some(scripts) = witness_set.plutus_scripts() {
        for i in 0..scripts.len() {
            let script = scripts.get(i);
            vkey_relation_map.add_relation(
                tx_id,
                RelationMap::scripthash_to_pallas(&script.hash(ScriptHashNamespace::PlutusV1))
                    .as_slice(),
                TxCredentialRelationValue::Witness,
            );
        }
    }
}
