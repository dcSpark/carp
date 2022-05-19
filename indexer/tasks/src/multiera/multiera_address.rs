use std::collections::{BTreeMap, BTreeSet};

use cardano_multiplatform_lib::address::{
    BaseAddress, ByronAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::primitives::alonzo::{
    self, Certificate, TransactionBody, TransactionBodyComponent,
};
use pallas::ledger::primitives::Fragment;
use std::ops::Deref;

use crate::{
    dsl::default_impl::has_transaction_multiera,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
};

use super::{
    multiera_address_credential_relations::QueuedAddressCredentialRelation,
    multiera_txs::MultieraTransactionTask, relation_map::RelationMap,
};
use crate::dsl::default_impl::EmptyConfiguration;

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraAddressTask;
  configuration EmptyConfiguration;
  doc "Adds the address raw bytes to the database";
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_txs];
  write [vkey_relation_map, multiera_addresses, multiera_queued_addresses_relations];
  should_add_task |block, _properties| {
    // recall: txs may have no outputs if they just burn all inputs as fee
    // TODO: this runs slightly more than it should
    has_transaction_multiera(block.1)
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
    block: BlockInfo<'_, alonzo::Block>,
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
    let mut queued_address = BTreeSet::<Vec<u8>>::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            match component {
                TransactionBodyComponent::Certificates(certs) => {
                    for cert in certs.iter() {
                        queue_certificate(
                            vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            cardano_transaction.id,
                            cert,
                        );
                    }
                }
                TransactionBodyComponent::Outputs(outputs) => {
                    for output in outputs.iter() {
                        queue_output(
                            vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            tx_body,
                            cardano_transaction.id,
                            output,
                        );
                    }
                }
                TransactionBodyComponent::Withdrawals(withdrawal_pairs) => {
                    for pair in withdrawal_pairs.deref() {
                        let reward_addr = RewardAddress::from_address(
                            &cardano_multiplatform_lib::address::Address::from_bytes(
                                pair.0.clone().into(),
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        queue_address_credential(
                            vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            cardano_transaction.id,
                            &reward_addr.to_address().to_bytes(),
                            &reward_addr.payment_cred(),
                            TxCredentialRelationValue::Withdrawal,
                            AddressCredentialRelationValue::PaymentKey,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let addresses = crate::era_common::insert_addresses(&queued_address, db_tx).await?;

    Ok((addresses, queued_address_credential))
}

fn queue_certificate(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    tx_id: i64,
    cert: &Certificate,
) {
    match cert {
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
                    &cardano_multiplatform_lib::crypto::Ed25519KeyHash::from_bytes(pool.to_vec())
                        .unwrap(),
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
                &cardano_multiplatform_lib::address::Address::from_bytes(reward_account.to_vec())
                    .unwrap(),
            )
            .unwrap();

            queue_address_credential(
                vkey_relation_map,
                queued_address_credential,
                queued_address,
                tx_id,
                &reward_addr.to_address().to_bytes(),
                &reward_addr.payment_cred(),
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

fn queue_output(
    queued_credentials: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    tx_body: &TransactionBody,
    tx_id: i64,
    output: &alonzo::TransactionOutput,
) {
    use cardano_multiplatform_lib::address::Address;
    let addr = Address::from_bytes(output.address.to_vec())
        .map_err(|e| panic!("{:?}{:?}", e, tx_body.to_hash().to_vec()))
        .unwrap();

    let tx_relation = TxCredentialRelationValue::Output;
    let address_relation = AddressCredentialRelationValue::PaymentKey;

    if let Some(base_addr) = BaseAddress::from_address(&addr) {
        // Payment Key
        {
            queue_address_credential(
                queued_credentials,
                queued_address_credential,
                queued_address,
                tx_id,
                &addr.to_bytes(),
                &base_addr.payment_cred(),
                tx_relation,
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
                &addr.to_bytes(),
                &base_addr.stake_cred(),
                TxCredentialRelationValue::OutputStake,
                AddressCredentialRelationValue::StakeKey,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &reward_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else if ByronAddress::from_address(&addr).is_some() {
        queued_address.insert(addr.to_bytes());
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &enterprise_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &ptr_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else {
        panic!("Unexpected address type {}", hex::encode(addr.to_bytes()));
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_address_credential(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredentialRelation>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    tx_id: i64,
    address: &[u8],
    credential: &cardano_multiplatform_lib::address::StakeCredential,
    tx_relation: TxCredentialRelationValue,
    address_relation: AddressCredentialRelationValue,
) {
    queued_address.insert(address.to_vec());
    vkey_relation_map.add_relation(tx_id, &credential.to_bytes(), tx_relation);
    queued_address_credential.insert(QueuedAddressCredentialRelation {
        address: address.to_vec(),
        stake_credential: credential.to_bytes(),
        address_relation,
    });
}
