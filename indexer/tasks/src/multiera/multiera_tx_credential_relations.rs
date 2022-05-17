extern crate shred;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::alonzo::{self};

use super::{
    multiera_address::MultieraAddressTask, multiera_stake_credentials::MultieraStakeCredentialTask,
    relation_map::RelationMap,
};

use crate::{dsl::default_impl::has_transaction_multiera, dsl::task_macro::*};

carp_task! {
  name MultieraTxCredentialRelationTask;
  doc "Adds the relation between transactions and credentials that appear within the tx to the database";
  era multiera;
  dependencies [MultieraAddressTask, MultieraStakeCredentialTask];
  read [multiera_stake_credential, vkey_relation_map];
  write [];
  should_add_task |block, _properties| {
    has_transaction_multiera(block.1)
  };
  execute |previous_data, task| handle_tx_credential_relations(
      task.db_tx,
      &previous_data.multiera_stake_credential,
      &previous_data.vkey_relation_map,
  );
  merge_result |previous_data, _result| {
  };
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
