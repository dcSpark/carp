use std::collections::{BTreeMap, BTreeSet};

use cardano_multiplatform_lib::{utils::ScriptHashNamespace, RequiredSignersSet};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};

use crate::{dsl::default_impl::has_transaction_multiera, types::TxCredentialRelationValue};

use super::{
    multiera_unused_input::MultieraUnusedInputTask, multiera_used_inputs::MultieraUsedInputTask,
    relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::task_macro::*;
use pallas::ledger::primitives::Fragment;

carp_task! {
  name MultieraStakeCredentialTask;
  configuration EmptyConfig;
  doc "Adds the stake credentials to the database.
       Note: \"stake credentials\" are an unfortunately poorly named type in the Cardano binary specification.
       A stake credential has nothing to do with staking. It's just a hash with an prefix to specify what kind of hash it is (ex: payment vs script)";
  era multiera;
  dependencies [MultieraUsedInputTask, MultieraUnusedInputTask];
  read [multiera_txs];
  write [vkey_relation_map, multiera_stake_credential];
  should_add_task |block, _properties| {
    has_transaction_multiera(block.1)
  };
  execute |previous_data, task| handle_stake_credentials(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_stake_credential = result;
  };
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
