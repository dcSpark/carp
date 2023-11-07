use std::collections::{BTreeMap, BTreeSet};

use cml_chain::certs::Credential;
use cml_chain::{
    address::{BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    byron::ByronAddress,
};
use cml_core::serialization::{FromBytes, Serialize, ToBytes};
use cml_crypto::RawBytesEncoding;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::{
    primitives::{alonzo::Certificate, Fragment},
    traverse::{MultiEraBlock, MultiEraCert, MultiEraOutput, MultiEraTx},
};
use std::ops::Deref;

use crate::types::{AddressCredentialRelationValue, TxCredentialRelationValue};

use super::{
    multiera_address_credential_relations::QueuedAddressCredentialRelation,
    multiera_txs::MultieraTransactionTask, relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraAddressTask;
  configuration EmptyConfig;
  doc "Adds the address raw bytes to the database";
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_txs];
  write [vkey_relation_map, multiera_addresses, multiera_queued_addresses_relations];
  should_add_task |block, _properties| {
    // recall: txs may have no outputs if they just burn all inputs as fee
    // TODO: this runs slightly more than it should
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_addresses(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_addresses = result.0;
    *previous_data.multiera_queued_addresses_relations = result.1;
  };
}

async fn handle_addresses(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<
    (
        BTreeMap<Vec<u8>, AddressInBlock>,
        BTreeSet<QueuedAddressCredentialRelation>,
    ),
    DbErr,
> {
    let mut queued_address_credential = BTreeSet::<QueuedAddressCredentialRelation>::default();
    let mut queued_address = BTreeMap::<Vec<u8>, i64>::default();

    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        for cert in tx_body.certs() {
            queue_certificate(
                vkey_relation_map,
                &mut queued_address_credential,
                &mut queued_address,
                cardano_transaction.id,
                &cert,
            );
        }

        for output in tx_body.outputs() {
            queue_output(
                vkey_relation_map,
                &mut queued_address_credential,
                &mut queued_address,
                tx_body,
                cardano_transaction.id,
                &output,
                TxCredentialRelationValue::Output,
                TxCredentialRelationValue::OutputStake,
            );
        }

        if let Some(collateral_return) = tx_body.collateral_return().as_ref() {
            queue_output(
                vkey_relation_map,
                &mut queued_address_credential,
                &mut queued_address,
                tx_body,
                cardano_transaction.id,
                collateral_return,
                TxCredentialRelationValue::UnusedOutput,
                TxCredentialRelationValue::UnusedOutputStake,
            );
        }

        for withdrawal in tx_body.withdrawals().collect::<Vec<(&[u8], u64)>>() {
            let reward_addr = RewardAddress::from_address(
                &cml_chain::address::Address::from_bytes(withdrawal.0.into()).unwrap(),
            )
            .unwrap();
            queue_address_credential(
                vkey_relation_map,
                &mut queued_address_credential,
                &mut queued_address,
                cardano_transaction.id,
                &reward_addr.clone().to_address().to_raw_bytes(),
                reward_addr.payment,
                TxCredentialRelationValue::Withdrawal,
                AddressCredentialRelationValue::PaymentKey,
            );
        }
    }

    let addresses = crate::era_common::insert_addresses(&queued_address, db_tx).await?;

    Ok((addresses, queued_address_credential))
}

fn queue_certificate(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeMap<Vec<u8>, i64>,
    tx_id: i64,
    cert: &MultiEraCert,
) {
    // TODO: what's the policy for handling options? At the moment of writing, all certificates
    // are "alonzo-compatible", but that might change in a future HF. Should Carp skip data that
    // it doesn't understand or instead panic? For now, opting to panic as it seems to be what's
    // used for other options.
    match cert.as_alonzo().unwrap() {
        Certificate::StakeDelegation(credential, pool) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeDelegation,
            );

            vkey_relation_map.add_relation(
                tx_id,
                RelationMap::keyhash_to_pallas(
                    cml_chain::crypto::Ed25519KeyHash::from_raw_bytes(pool.as_slice()).unwrap(),
                )
                .as_slice(),
                TxCredentialRelationValue::DelegationTarget,
            );
        }
        Certificate::StakeRegistration(credential) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeRegistration,
            );
        }
        Certificate::StakeDeregistration(credential) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeDeregistration,
            );
        }
        Certificate::PoolRegistration {
            operator,
            pool_owners,
            reward_account,
            ..
        } => {
            let operator_credential =
                pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(*operator)
                    .encode_fragment()
                    .unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &operator_credential,
                TxCredentialRelationValue::PoolOperator,
            );

            let reward_addr = RewardAddress::from_address(
                &cml_chain::address::Address::from_bytes(reward_account.to_vec()).unwrap(),
            )
            .unwrap();

            queue_address_credential(
                vkey_relation_map,
                queued_address_credential,
                queued_address,
                tx_id,
                &reward_addr.clone().to_address().to_raw_bytes(),
                reward_addr.payment,
                TxCredentialRelationValue::PoolReward,
                AddressCredentialRelationValue::PaymentKey,
            );

            for &owner in pool_owners.iter() {
                let owner_credential =
                    pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(owner)
                        .encode_fragment()
                        .unwrap();

                vkey_relation_map.add_relation(
                    tx_id,
                    &owner_credential,
                    TxCredentialRelationValue::PoolOwner,
                );
            }
        }
        Certificate::PoolRetirement(key_hash, _) => {
            let operator_credential =
                pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(*key_hash)
                    .encode_fragment()
                    .unwrap();
            vkey_relation_map.add_relation(
                tx_id,
                &operator_credential,
                TxCredentialRelationValue::PoolOperator,
            );
        }
        Certificate::GenesisKeyDelegation(_, _, _) => {
            // genesis keys aren't stake credentials
        }
        Certificate::MoveInstantaneousRewardsCert(mir) => {
            if let pallas::ledger::primitives::alonzo::InstantaneousRewardTarget::StakeCredentials(
                credential_pairs,
            ) = &mir.target
            {
                for pair in credential_pairs.deref() {
                    let credential = pair.0.encode_fragment().unwrap();

                    vkey_relation_map.add_relation(
                        tx_id,
                        &credential,
                        TxCredentialRelationValue::MirRecipient,
                    );
                }
            }
        }
    };
}

#[allow(clippy::too_many_arguments)]
fn queue_output(
    queued_credentials: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeMap<Vec<u8>, i64>,
    tx_body: &MultiEraTx,
    tx_id: i64,
    output: &MultiEraOutput,
    output_relation: TxCredentialRelationValue,
    output_stake_relation: TxCredentialRelationValue,
) {
    use cml_chain::address::Address;

    let pallas_address = output
        .address()
        .map_err(|e| panic!("{:?} {:?}", e, hex::encode(tx_body.hash())))
        .unwrap();
    let addr = Address::from_bytes(pallas_address.to_vec())
        .map_err(|e| panic!("{:?} {:?}", e, hex::encode(tx_body.hash())))
        .unwrap();

    let address_relation = AddressCredentialRelationValue::PaymentKey;

    if let Some(base_addr) = BaseAddress::from_address(&addr) {
        // Payment Key
        {
            queue_address_credential(
                queued_credentials,
                queued_address_credential,
                queued_address,
                tx_id,
                &addr.to_raw_bytes(),
                base_addr.payment,
                output_relation,
                address_relation,
            );
        }

        // Stake Key
        {
            queue_address_credential(
                queued_credentials,
                queued_address_credential,
                queued_address,
                tx_id,
                &addr.to_raw_bytes(),
                base_addr.stake,
                output_stake_relation,
                AddressCredentialRelationValue::StakeKey,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_raw_bytes(),
            reward_addr.payment,
            output_relation,
            address_relation,
        );
    } else if ByronAddress::from_address(&addr).is_some() {
        queued_address
            .entry(addr.to_raw_bytes())
            .and_modify(|old_id| {
                if tx_id < *old_id {
                    *old_id = tx_id
                }
            })
            .or_insert(tx_id);
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_raw_bytes(),
            enterprise_addr.payment,
            output_relation,
            address_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_raw_bytes(),
            ptr_addr.payment,
            output_relation,
            address_relation,
        );
    } else {
        panic!(
            "Unexpected address type {}",
            hex::encode(addr.to_raw_bytes())
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_address_credential(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeMap<Vec<u8>, i64>,
    tx_id: i64,
    address: &[u8],
    credential: Credential,
    tx_relation: TxCredentialRelationValue,
    address_relation: AddressCredentialRelationValue,
) {
    queued_address
        .entry(address.to_vec())
        .and_modify(|old_id| {
            if tx_id < *old_id {
                *old_id = tx_id
            }
        })
        .or_insert(tx_id);
    vkey_relation_map.add_relation(tx_id, credential.to_raw_bytes(), tx_relation);
    queued_address_credential.insert(QueuedAddressCredentialRelation {
        address: address.to_vec(),
        stake_credential: credential.to_raw_bytes().to_vec(),
        address_relation,
    });
}
