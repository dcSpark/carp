use cml_chain::transaction::utils::RequiredSignersSet;
use cml_chain::transaction::TransactionWitnessSet;
use cml_core::serialization::FromBytes;
use cml_crypto::RawBytesEncoding;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};

use crate::types::TxCredentialRelationValue;

use super::{
    multiera_unused_input::MultieraUnusedInputTask, multiera_used_inputs::MultieraUsedInputTask,
    relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;

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
    !block.1.is_empty()
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
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    for ((tx_body, tx_witness), cardano_transaction) in block
        .1
        .transaction_bodies()
        .iter()
        .zip(block.1.transaction_witness_sets().iter())
        .zip(multiera_txs)
    {
        queue_witness(
            vkey_relation_map,
            cardano_transaction.id,
            tx_witness.clone(),
        );

        for signer in tx_body.required_signers().cloned().unwrap_or_default() {
            let owner_credential = cml_chain::certs::Credential::new_pub_key(signer)
                .to_raw_bytes()
                .to_vec();
            vkey_relation_map.add_relation(
                cardano_transaction.id,
                &owner_credential,
                TxCredentialRelationValue::RequiredSigner,
            );
        }
    }

    let mut queued_cred = BTreeMap::<Vec<u8>, i64>::default();
    for (tx_id, cred_to_relation) in &vkey_relation_map.0 {
        for key in cred_to_relation.keys() {
            // we want to keep track of the first tx for each credential
            queued_cred.entry(key.to_vec()).or_insert(*tx_id);
        }
    }
    let cred_to_model_map = insert_stake_credentials(&queued_cred, db_tx).await?;
    Ok(cred_to_model_map)
}

async fn insert_stake_credentials(
    deduplicated_creds: &BTreeMap<Vec<u8>, i64>,
    txn: &DatabaseTransaction,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    let mut result_map = BTreeMap::<Vec<u8>, StakeCredentialModel>::default();

    if deduplicated_creds.is_empty() {
        return Ok(result_map);
    }

    // 1) Add credentials that were already in the DB
    {
        let mut found_credentials =
            StakeCredential::find()
                .filter(Condition::any().add(
                    StakeCredentialColumn::Credential.is_in(deduplicated_creds.keys().cloned()),
                ))
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
        let mut credentials_to_add: Vec<StakeCredentialActiveModel> = deduplicated_creds
            .iter()
            .filter(|&(credential, _)| !result_map.contains_key(credential))
            .map(|(credential, tx_id)| StakeCredentialActiveModel {
                credential: Set(credential.to_vec()),
                first_tx: Set(*tx_id),
                ..Default::default()
            })
            .collect();

        // need to make sure we're inserting addresses in the same order as we added txs
        credentials_to_add.sort_by(|a, b| a.first_tx.as_ref().cmp(b.first_tx.as_ref()));

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
    witness_set: TransactionWitnessSet,
) {
    if let Some(vkeys) = witness_set.vkeywitnesses {
        for vkey in vkeys {
            vkey_relation_map.add_relation(
                tx_id,
                vkey.vkey.hash().to_raw_bytes(),
                TxCredentialRelationValue::Witness,
            );
        }
    }
    if let Some(scripts) = witness_set.native_scripts {
        for script in scripts {
            vkey_relation_map.add_relation(
                tx_id,
                script.hash().to_raw_bytes(),
                TxCredentialRelationValue::Witness,
            );

            let vkeys_in_script = RequiredSignersSet::from(&script);
            for vkey_in_script in vkeys_in_script {
                vkey_relation_map.add_relation(
                    tx_id,
                    vkey_in_script.to_raw_bytes(),
                    TxCredentialRelationValue::InNativeScript,
                );
            }
        }
    }

    if let Some(scripts) = &witness_set.plutus_v1_scripts {
        for script in scripts {
            vkey_relation_map.add_relation(
                tx_id,
                script.hash().to_raw_bytes(),
                TxCredentialRelationValue::Witness,
            );
        }
    }
    if let Some(scripts) = &witness_set.plutus_v2_scripts {
        for script in scripts {
            vkey_relation_map.add_relation(
                tx_id,
                script.hash().to_raw_bytes(),
                TxCredentialRelationValue::Witness,
            );
        }
    }
}
