extern crate shred;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use cardano_multiplatform_lib::address::{
    BaseAddress, ByronAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{
    self, Certificate, TransactionBody, TransactionBodyComponent,
};
use pallas::ledger::primitives::Fragment;
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::ops::Deref;

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    era_common::AddressInBlock,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
    utils::TaskPerfAggregator,
};

use super::{
    multiera_address_credential_relations::QueuedAddressCredentialRelation,
    multiera_txs::MultieraTransactionTask, relation_map::RelationMap,
};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    vkey_relation_map: Write<'a, RelationMap>,
    multiera_addresses: Write<'a, BTreeMap<Vec<u8>, AddressInBlock>>,
    multiera_queued_addresses_relations: Write<'a, BTreeSet<QueuedAddressCredentialRelation>>,
}

pub struct MultieraAddressTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraAddressTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraAddressTask);
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(MultieraTransactionTask)];

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

struct MultieraAddressTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraAddressTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraAddressTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraAddressTask::DEPENDENCIES
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
        let task = MultieraAddressTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraAddressTaskBuilder })
}

impl<'a> System<'a> for MultieraAddressTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_addresses(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &mut bundle.vkey_relation_map,
            ))
            .unwrap();
        *bundle.multiera_addresses = result.0;
        *bundle.multiera_queued_addresses_relations = result.1;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
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
