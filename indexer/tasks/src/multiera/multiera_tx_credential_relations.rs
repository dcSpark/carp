extern crate shred;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cardano_multiplatform_lib::{utils::ScriptHashNamespace, RequiredSignersSet};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
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
use pallas::ledger::primitives::Fragment;

use super::{
    multiera_address::MultieraAddressTask, multiera_stake_credentials::MultieraStakeCredentialTask,
    relation_map::RelationMap,
};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    multiera_stake_credential: Read<'a, BTreeMap<Vec<u8>, StakeCredentialModel>>,
    vkey_relation_map: Write<'a, RelationMap>,
}

pub struct MultieraTxCredentialRelation<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraTxCredentialRelation<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraTxCredentialRelation);
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

struct MultieraTxCredentialRelationBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraTxCredentialRelationBuilder {
    fn get_name(&self) -> &'static str {
        MultieraTxCredentialRelation::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraTxCredentialRelation::DEPENDENCIES
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
        let task = MultieraTxCredentialRelation::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraTxCredentialRelationBuilder })
}

impl<'a> System<'a> for MultieraTxCredentialRelation<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        self.handle
            .block_on(handle_tx_credential_relations(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &bundle.multiera_stake_credential,
                &mut bundle.vkey_relation_map,
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
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    vkey_relation_map: &mut RelationMap,
) -> Result<(), DbErr> {
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
    insert_tx_credentials(vkey_relation_map, multiera_stake_credential, db_tx).await?;

    Ok(())
}

async fn insert_tx_credentials(
    vkey_relation_map: &RelationMap,
    cred_to_model_map: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let mut models: Vec<TxCredentialActiveModel> = vec![];
    for (tx_id, mapping) in vkey_relation_map.0.iter() {
        models.extend(
            mapping
                .iter()
                .map(|(credential, relation)| TxCredentialActiveModel {
                    credential_id: Set(cred_to_model_map.get(&credential.to_vec()).unwrap().id),
                    tx_id: Set(*tx_id),
                    relation: Set(*relation),
                }),
        );
    }

    // no entries to add if tx only had Byron addresses
    if !models.is_empty() {
        TxCredential::insert_many(models).exec(txn).await?;
    }
    Ok(())
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
