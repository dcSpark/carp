use std::collections::BTreeSet;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::{dsl::default_impl::has_transaction_multiera, types::AddressCredentialRelationValue};
use entity::sea_orm::Condition;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};

use crate::dsl::task_macro::*;

use super::{
    multiera_address::MultieraAddressTask, multiera_stake_credentials::MultieraStakeCredentialTask,
};

carp_task! {
  name MultieraAddressCredentialRelationTask;
  configuration ReadonlyConfig;
  doc "Adds to the database the relation between addresses and the credentials part of the addresses (ex: payment key + staking key)";
  era multiera;
  dependencies [MultieraAddressTask, MultieraStakeCredentialTask];
  read [multiera_addresses, multiera_queued_addresses_relations, multiera_stake_credential];
  write [];
  should_add_task |block, _properties| {
    has_transaction_multiera(block.1)
  };
  execute |previous_data, task| handle_address_credential_relation(
      task.db_tx,
      &previous_data.multiera_stake_credential,
      &previous_data.multiera_addresses,
      &previous_data.multiera_queued_addresses_relations,
      task.config.readonly
  );
  merge_result |previous_data, _result| {
  };
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
    readonly: bool,
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
        false => {
            if readonly {
                Ok(to_add
                    .iter()
                    .map(|entry| AddressCredentialModel {
                        credential_id: entry.credential_id.clone().unwrap(),
                        address_id: entry.address_id.clone().unwrap(),
                        relation: entry.relation.clone().unwrap(),
                    })
                    .collect())
            } else {
                Ok(AddressCredential::insert_many(to_add.clone())
                    .exec_many_with_returning(db_tx)
                    .await?)
            }
        }
    }
}
