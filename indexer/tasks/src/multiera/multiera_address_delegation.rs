use crate::{
    multiera::multiera_stake_credentials::MultieraStakeCredentialTask,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
};
use cml_core::serialization::Serialize;
use cml_crypto::RawBytesEncoding;
use cml_multi_era::utils::MultiEraCertificate;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use sea_orm::{Order, QueryOrder, Set};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

use super::{
    multiera_address_credential_relations::QueuedAddressCredentialRelation,
    multiera_txs::MultieraTransactionTask, relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;

carp_task! {
  name MultieraAddressDelegationTask;
  configuration EmptyConfig;
  doc "Tracks stake delegation actions to pools.";
  era multiera;
  dependencies [MultieraStakeCredentialTask];
  read [multiera_txs, multiera_stake_credential];
  write [];
  should_add_task |block, _properties| {
    // recall: txs may have no outputs if they just burn all inputs as fee
    // TODO: this runs slightly more than it should
    !block.1.is_empty()
  };
  execute |previous_data, task| handle(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_stake_credential,
  );
  merge_result |_previous_data, _result| {};
}

async fn handle(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
) -> Result<(), DbErr> {
    for (tx_body, cardano_transaction) in block.1.transaction_bodies().iter().zip(multiera_txs) {
        let certs = match tx_body.certs() {
            None => continue,
            Some(certs) => certs,
        };
        for cert in certs {
            {
                let tx_id = cardano_transaction.id;
                let cert = &cert;
                let (credential, pool) = match cert {
                    MultiEraCertificate::StakeDelegation(delegation) => {
                        (delegation.stake_credential.clone(), Some(delegation.pool))
                    }
                    MultiEraCertificate::StakeDeregistration(deregistration) => {
                        (deregistration.stake_credential.clone(), None)
                    }
                    _ => continue,
                };

                let credential = credential.to_cbor_bytes();

                let stake_credential_id = multiera_stake_credential
                    .get(&credential.to_vec())
                    .unwrap()
                    .id;

                let previous_entry = entity::stake_delegation::Entity::find()
                    .filter(
                        entity::stake_delegation::Column::StakeCredential.eq(stake_credential_id),
                    )
                    .order_by_desc(entity::stake_delegation::Column::Id)
                    .one(db_tx)
                    .await?;

                let pool = pool.map(|pool| pool.to_raw_bytes().to_vec());

                if let Some((previous, pool)) = previous_entry
                    .as_ref()
                    .and_then(|entry| entry.pool_credential.as_ref())
                    .zip(pool.as_ref())
                {
                    // re-delegating shouldn't have any effect.
                    if previous == pool {
                        continue;
                    }
                }

                entity::stake_delegation::ActiveModel {
                    stake_credential: Set(stake_credential_id),
                    pool_credential: Set(pool.map(|pool| pool.to_vec())),
                    tx_id: Set(tx_id),
                    previous_pool: Set(previous_entry.and_then(|entity| entity.pool_credential)),
                    ..Default::default()
                }
                .save(db_tx)
                .await?;
            };
        }
    }

    Ok(())
}
