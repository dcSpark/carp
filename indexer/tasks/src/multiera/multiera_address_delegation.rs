use crate::{
    multiera::multiera_stake_credentials::MultieraStakeCredentialTask,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
};
use cardano_multiplatform_lib::{
    address::{BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    byron::ByronAddress,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::{
    primitives::{alonzo::Certificate, Fragment},
    traverse::{MultiEraBlock, MultiEraCert, MultiEraOutput, MultiEraTx},
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
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
) -> Result<(), DbErr> {
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        for cert in tx_body.certs() {
            {
                let tx_id = cardano_transaction.id;
                let cert = &cert;
                match cert.as_alonzo().unwrap() {
                    Certificate::StakeDelegation(credential, pool) => {
                        let credential = credential.encode_fragment().unwrap();

                        let stake_credential_id = multiera_stake_credential
                            .get(&credential.to_vec())
                            .unwrap()
                            .id;

                        entity::stake_delegation::ActiveModel {
                            stake_credential: Set(stake_credential_id),
                            pool_credential: Set(Some(pool.to_vec())),
                            tx_id: Set(tx_id),
                            ..Default::default()
                        }
                        .save(db_tx)
                        .await?;
                    }
                    Certificate::StakeRegistration(credential) => {}
                    Certificate::StakeDeregistration(credential) => {
                        let credential = credential.encode_fragment().unwrap();

                        let stake_credential_id = multiera_stake_credential
                            .get(&credential.to_vec())
                            .unwrap()
                            .id;

                        entity::stake_delegation::ActiveModel {
                            stake_credential: Set(stake_credential_id),
                            pool_credential: Set(None),
                            tx_id: Set(tx_id),
                            ..Default::default()
                        }
                        .save(db_tx)
                        .await?;
                    }
                    Certificate::PoolRegistration {
                        operator,
                        pool_owners,
                        reward_account,
                        ..
                    } => {}
                    Certificate::PoolRetirement(key_hash, _) => {}
                    Certificate::GenesisKeyDelegation(_, _, _) => {}
                    Certificate::MoveInstantaneousRewardsCert(mir) => {}
                };
            };
        }
    }

    Ok(())
}
