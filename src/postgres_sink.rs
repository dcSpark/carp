use std::ops::Deref;

use anyhow::anyhow;
use cardano_serialization_lib::address::{
    BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use cryptoxide::blake2b::Blake2b;
use oura::{
    model::{BlockRecord, Era, EventData},
    pipelining::StageReceiver,
};
use pallas::{
    crypto::hash::Hash,
    ledger::primitives::{
        alonzo::{self, Certificate, TransactionBodyComponent},
        byron::{self, TxIn},
        Fragment,
    },
};

use crate::types::{AddressCredentialRelationValue, MultiEraBlock, TxCredentialRelationValue};
use entity::{
    prelude::*,
    sea_orm::{
        prelude::*, ColumnTrait, DatabaseTransaction, JoinType, QuerySelect, Set, TransactionTrait,
    },
};
use migration::DbErr;
use std::time::Duration;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct PerfAggregator {
    block_insertion: Duration,
    transaction_insert: Duration,
    transaction_input_insert: Duration,
    transaction_output_insert: Duration,
    certificate_insert: Duration,
    collateral_insert: Duration,
    overhead: Duration,
}
impl PerfAggregator {
    pub fn new() -> Self {
        Self {
            block_insertion: Duration::new(0, 0),
            transaction_insert: Duration::new(0, 0),
            transaction_input_insert: Duration::new(0, 0),
            transaction_output_insert: Duration::new(0, 0),
            certificate_insert: Duration::new(0, 0),
            collateral_insert: Duration::new(0, 0),
            overhead: Duration::new(0, 0),
        }
    }
    pub fn set_overhead(&mut self, total_duration: &Duration) {
        let non_duration_sum = self.block_insertion
            + self.transaction_insert
            + self.transaction_input_insert
            + self.transaction_output_insert
            + self.certificate_insert
            + self.collateral_insert;
        self.overhead = *total_duration - non_duration_sum
    }
}
impl std::ops::Add for PerfAggregator {
    type Output = PerfAggregator;

    fn add(self, other: Self) -> Self {
        Self {
            block_insertion: self.block_insertion + other.block_insertion,
            transaction_insert: self.transaction_insert + other.transaction_insert,
            transaction_input_insert: self.transaction_input_insert
                + other.transaction_input_insert,
            transaction_output_insert: self.transaction_output_insert
                + other.transaction_output_insert,
            certificate_insert: self.certificate_insert + other.certificate_insert,
            collateral_insert: self.collateral_insert + other.collateral_insert,
            overhead: self.collateral_insert + other.overhead,
        }
    }
}
impl std::ops::AddAssign for PerfAggregator {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl<'a> Config<'a> {
    pub async fn bootstrap(&self, input: StageReceiver) -> anyhow::Result<()> {
        tracing::info!("{}", "Starting to process blocks");

        let mut last_epoch: i128 = -1;
        let mut epoch_start_time = std::time::Instant::now();
        let mut perf_aggregator = PerfAggregator::new();

        loop {
            let event = input.recv()?;

            let data = event.data.clone();

            match data {
                EventData::Block(block_record) => {
                    match block_record.epoch {
                        Some(epoch) if epoch as i128 > last_epoch => {
                            let epoch_duration = epoch_start_time.elapsed();
                            tracing::info!(
                                "Finished processing epoch {} after {:?}",
                                epoch,
                                epoch_duration
                            );
                            perf_aggregator.set_overhead(&epoch_duration);
                            tracing::trace!("Epoch part-wise time spent:\n{:#?}", perf_aggregator);
                            epoch_start_time = std::time::Instant::now();
                            perf_aggregator = PerfAggregator::new();

                            tracing::info!(
                                "Starting epoch {} at block #{} ({})",
                                epoch,
                                block_record.number,
                                block_record.hash
                            );
                            last_epoch = epoch as i128;
                        }
                        _ => (),
                    };
                    perf_aggregator += self
                        .conn
                        .transaction::<_, PerfAggregator, DbErr>(|txn| {
                            Box::pin(insert(block_record, txn))
                        })
                        .await?;
                }
                EventData::RollBack { block_slot, .. } => {
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gt(block_slot))
                        .exec(self.conn)
                        .await?;
                    match block_slot {
                        0 => tracing::info!("Rollback to genesis"),
                        _ => tracing::info!("Rollback to slot {}", block_slot - 1),
                    }
                }
                _ => (),
            }
        }
    }
}

fn blake2b256(data: &[u8]) -> [u8; 32] {
    let mut out = [0; 32];
    Blake2b::blake2b(&mut out, data, &[]);
    out
}

async fn insert(
    block_record: BlockRecord,
    txn: &DatabaseTransaction,
) -> Result<PerfAggregator, DbErr> {
    let hash = hex::decode(&block_record.hash).unwrap();
    let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();

    let mut perf_aggregator = PerfAggregator::new();
    let mut time_counter = std::time::Instant::now();

    let (multi_block, era) = block_with_era(block_record.era, &block_payload).unwrap();

    let block = BlockActiveModel {
        era: Set(era),
        hash: Set(hash),
        height: Set(block_record.number as i64),
        epoch: Set(0),
        slot: Set(block_record.slot as i64),
        ..Default::default()
    };

    let block = block.insert(txn).await?;

    perf_aggregator.block_insertion += time_counter.elapsed();
    time_counter = std::time::Instant::now();

    match multi_block {
        MultiEraBlock::Byron(byron_block) => match byron_block.deref() {
            // Byron era had Epoch-boundary blocks for calculating stake distribution changes
            // they don't contain any txs, so we can just ignore them
            byron::Block::EbBlock(_) => (),
            byron::Block::MainBlock(main_block) => {
                for (idx, tx_body) in main_block.body.tx_payload.iter().enumerate() {
                    let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                    let tx_payload = tx_body.encode_fragment().unwrap();

                    let transaction = TransactionActiveModel {
                        hash: Set(tx_hash.to_vec()),
                        block_id: Set(block.id),
                        tx_index: Set(idx as i32),
                        payload: Set(tx_payload),
                        is_valid: Set(true),
                        ..Default::default()
                    };

                    let transaction = transaction.insert(txn).await?;

                    perf_aggregator.transaction_insert += time_counter.elapsed();
                    time_counter = std::time::Instant::now();

                    for (idx, output) in tx_body.transaction.outputs.iter().enumerate() {
                        let mut address_payload = output.address.encode_fragment().unwrap();

                        let address = insert_address(&mut address_payload, txn).await?;

                        let tx_output = TransactionOutputActiveModel {
                            payload: Set(output.encode_fragment().unwrap()),
                            address_id: Set(address.id),
                            tx_id: Set(transaction.id),
                            output_index: Set(idx as i64),
                            ..Default::default()
                        };

                        tx_output.save(txn).await?;
                    }

                    perf_aggregator.transaction_output_insert += time_counter.elapsed();
                    time_counter = std::time::Instant::now();

                    for (idx, input) in tx_body.transaction.inputs.iter().enumerate() {
                        let (tx_hash, index) = match input {
                            TxIn::Variant0(wrapped) => wrapped.deref(),
                            TxIn::Other(index, tx_hash) => {
                                todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
                            }
                        };

                        insert_input(&transaction, idx as i32, *index as u64, tx_hash, txn).await?;
                    }

                    perf_aggregator.transaction_input_insert += time_counter.elapsed();
                    time_counter = std::time::Instant::now();
                }
            }
        },
        MultiEraBlock::Compatible(alonzo_block) => {
            for (idx, (tx_body, tx_witness_set)) in alonzo_block
                .deref()
                .transaction_bodies
                .iter()
                .zip(alonzo_block.transaction_witness_sets.iter())
                .enumerate()
            {
                let tx_hash = alonzo::crypto::hash_transaction(tx_body).to_vec();

                let body_payload = tx_body.encode_fragment().unwrap();
                let body = &cardano_serialization_lib::TransactionBody::from_bytes(body_payload)
                    .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                    .unwrap();

                let witness_set_payload = tx_witness_set.encode_fragment().unwrap();
                let witness_set = &cardano_serialization_lib::TransactionWitnessSet::from_bytes(
                    witness_set_payload,
                )
                .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                .unwrap();

                let aux_data = alonzo_block
                    .auxiliary_data_set
                    .iter()
                    .find(|(index, _)| *index as usize == idx);

                let auxiliary_data = aux_data.map(|(_, a)| {
                    let auxiliary_data_payload = a.encode_fragment().unwrap();

                    cardano_serialization_lib::metadata::AuxiliaryData::from_bytes(
                        auxiliary_data_payload,
                    )
                    .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                    .unwrap()
                });

                let mut temp_tx =
                    cardano_serialization_lib::Transaction::new(body, witness_set, auxiliary_data);

                let mut is_valid = true;

                if let Some(ref invalid_txs) = alonzo_block.invalid_transactions {
                    is_valid = !invalid_txs.iter().any(|i| *i as usize == idx)
                }

                temp_tx.set_is_valid(is_valid);

                let transaction = TransactionActiveModel {
                    hash: Set(tx_hash),
                    block_id: Set(block.id),
                    tx_index: Set(idx as i32),
                    payload: Set(temp_tx.to_bytes()),
                    is_valid: Set(is_valid),
                    ..Default::default()
                };

                let transaction = transaction.insert(txn).await?;

                perf_aggregator.transaction_insert += time_counter.elapsed();
                time_counter = std::time::Instant::now();

                for component in tx_body.iter() {
                    insert_certificates(&transaction, component, txn).await?;
                }

                perf_aggregator.certificate_insert += time_counter.elapsed();
                time_counter = std::time::Instant::now();

                for component in tx_body.iter() {
                    match component {
                        TransactionBodyComponent::Outputs(outputs) => {
                            for (idx, output) in outputs.iter().enumerate() {
                                use cardano_serialization_lib::address::Address;

                                let address =
                                    insert_address(&mut output.address.to_vec(), txn).await?;

                                let addr = Address::from_bytes(output.address.to_vec())
                                    .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                                    .unwrap();

                                let tx_relation = TxCredentialRelationValue::Output;
                                let address_relation = AddressCredentialRelationValue::PaymentKey;

                                if let Some(base_addr) = BaseAddress::from_address(&addr) {
                                    // Payment Key
                                    let payload = base_addr.payment_cred().to_bytes();

                                    insert_address_credential(
                                        payload,
                                        &transaction,
                                        &address,
                                        tx_relation.into(),
                                        address_relation.into(),
                                        txn,
                                    )
                                    .await?;

                                    // Stake Key
                                    let payload = base_addr.stake_cred().to_bytes();

                                    let address_relation = AddressCredentialRelationValue::StakeKey;

                                    insert_address_credential(
                                        payload,
                                        &transaction,
                                        &address,
                                        tx_relation.into(),
                                        address_relation.into(),
                                        txn,
                                    )
                                    .await?;
                                } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
                                    let payload = ptr_addr.payment_cred().to_bytes();

                                    insert_address_credential(
                                        payload,
                                        &transaction,
                                        &address,
                                        tx_relation.into(),
                                        address_relation.into(),
                                        txn,
                                    )
                                    .await?;
                                } else if let Some(enterprise_addr) =
                                    EnterpriseAddress::from_address(&addr)
                                {
                                    let payload = enterprise_addr.payment_cred().to_bytes();

                                    insert_address_credential(
                                        payload,
                                        &transaction,
                                        &address,
                                        tx_relation.into(),
                                        address_relation.into(),
                                        txn,
                                    )
                                    .await?;
                                } else if let Some(reward_addr) = RewardAddress::from_address(&addr)
                                {
                                    let payload = reward_addr.payment_cred().to_bytes();
                                    insert_address_credential(
                                        payload,
                                        &transaction,
                                        &address,
                                        tx_relation.into(),
                                        address_relation.into(),
                                        txn,
                                    )
                                    .await?;
                                };

                                let tx_output = TransactionOutputActiveModel {
                                    payload: Set(output.encode_fragment().unwrap()),
                                    address_id: Set(address.id),
                                    tx_id: Set(transaction.id),
                                    output_index: Set(idx as i64),
                                    ..Default::default()
                                };

                                tx_output.save(txn).await?;
                            }
                            perf_aggregator.transaction_output_insert += time_counter.elapsed();
                            time_counter = std::time::Instant::now();
                        }
                        TransactionBodyComponent::Inputs(inputs) if is_valid => {
                            for (idx, input) in inputs.iter().enumerate() {
                                insert_input(
                                    &transaction,
                                    idx as i32,
                                    input.index,
                                    &input.transaction_id,
                                    txn,
                                )
                                .await?;
                            }
                            perf_aggregator.transaction_input_insert += time_counter.elapsed();
                            time_counter = std::time::Instant::now();
                        }
                        TransactionBodyComponent::Collateral(inputs) if !is_valid => {
                            for (idx, input) in inputs.iter().enumerate() {
                                insert_input(
                                    &transaction,
                                    idx as i32,
                                    input.index,
                                    &input.transaction_id,
                                    txn,
                                )
                                .await?;
                            }
                            perf_aggregator.collateral_insert += time_counter.elapsed();
                            time_counter = std::time::Instant::now();
                        }

                        _ => (),
                    }
                }
            }
        }
    }

    Ok(perf_aggregator)
}

async fn insert_address(
    payload: &mut Vec<u8>,
    txn: &DatabaseTransaction,
) -> Result<AddressModel, DbErr> {
    // During the Byron era of Cardano,
    // Addresses had a feature where you could add extra metadata in them
    // The amount of metadata you could insert was not capped
    // So some addresses got generated which are really large
    // However, Postgres btree v4 has a maximum size of 2704 for an index
    // Since these addresses can't be spent anyway, we just truncate them
    // theoretically, we could truncate at 2704, but we truncate at 500
    // reasons:
    // 1) Postgres has shrunk the limit in the past, so they may do it again
    // 2) Use of the INCLUDE in creating an index can increase its size
    //    So best to leave some extra room incase this is useful someday
    // 3) It's not great to hard-code a postgresql-specific limitation
    // 4) 500 seems more obviously human than 2704 so maybe easier if somebody sees it
    // 5) Storing up to 2704 bytes is a waste of space since they aren't used for anything
    payload.truncate(500);

    let addr = Address::find()
        .filter(AddressColumn::Payload.eq(payload.clone()))
        .one(txn)
        .await?;

    if let Some(addr) = addr {
        Ok(addr)
    } else {
        let address = AddressActiveModel {
            payload: Set(payload.clone()),
            ..Default::default()
        };

        let address = address.insert(txn).await?;
        Ok(address)
    }
}

async fn insert_address_credential(
    payload: Vec<u8>,
    tx: &TransactionModel,
    address: &AddressModel,
    tx_relation: i32,
    address_relation: i32,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let stake_credential = insert_credential(tx, payload, txn, tx_relation).await?;

    let address_credential = AddressCredentialActiveModel {
        credential_id: Set(stake_credential.id),
        address_id: Set(address.id),
        relation: Set(address_relation),
        ..Default::default()
    };

    address_credential.save(txn).await?;

    Ok(())
}

async fn insert_input(
    tx: &TransactionModel,
    idx: i32,
    index: u64,
    tx_hash: &Hash<32>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    // 1) Get the UTXO this input is spending
    let tx_output = TransactionOutput::find()
        .inner_join(Transaction)
        .filter(TransactionOutputColumn::OutputIndex.eq(index))
        .filter(TransactionColumn::Hash.eq(tx_hash.to_vec()))
        .one(txn)
        .await?;

    let tx_output = tx_output.unwrap();

    // 2) Get the stake credential for the UTXO being spent
    let stake_credentials = StakeCredential::find()
        .inner_join(AddressCredential)
        .join(
            JoinType::InnerJoin,
            AddressCredentialRelation::Address.def(),
        )
        .join(
            JoinType::InnerJoin,
            AddressRelation::TransactionOutput.def(),
        )
        .filter(TransactionOutputColumn::Id.eq(tx_output.id))
        .all(txn)
        .await?;

    // 3) Associate the stake credential to this transaction
    let relation = TxCredentialRelationValue::Input;
    for stake_credential in stake_credentials {
        let tx_credential = TxCredentialActiveModel {
            credential_id: Set(stake_credential.id),
            tx_id: Set(tx.id),
            relation: Set(relation.into()),
            ..Default::default()
        };

        tx_credential.save(txn).await?;
    }

    // 4) Add input itself
    let tx_input = TransactionInputActiveModel {
        utxo_id: Set(tx_output.id),
        tx_id: Set(tx.id),
        input_index: Set(idx),
        ..Default::default()
    };

    tx_input.save(txn).await?;

    Ok(())
}

async fn insert_certificates(
    tx: &TransactionModel,
    component: &TransactionBodyComponent,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if let TransactionBodyComponent::Certificates(certs) = component {
        for cert in certs.iter() {
            let (credential, relation) = match cert {
                Certificate::StakeDelegation(credential, _) => {
                    (credential, TxCredentialRelationValue::StakeDelegation)
                }
                Certificate::StakeRegistration(credential) => {
                    (credential, TxCredentialRelationValue::StakeRegistration)
                }
                Certificate::StakeDeregistration(credential) => {
                    (credential, TxCredentialRelationValue::StakeDeregistration)
                }
                _ => continue,
            };

            let credential = credential.encode_fragment().unwrap();

            insert_credential(tx, credential, txn, relation.into()).await?;
        }
    }

    Ok(())
}

async fn insert_credential(
    tx: &TransactionModel,
    credential: Vec<u8>,
    txn: &DatabaseTransaction,
    relation: i32,
) -> Result<StakeCredentialModel, DbErr> {
    let sc = StakeCredential::find()
        .filter(StakeCredentialColumn::Credential.eq(credential.clone()))
        .one(txn)
        .await?;

    if let Some(stake_credential) = sc {
        let tx_credential = TxCredentialActiveModel {
            credential_id: Set(stake_credential.id),
            tx_id: Set(tx.id),
            relation: Set(relation),
            ..Default::default()
        };

        tx_credential.save(txn).await?;

        Ok(stake_credential)
    } else {
        let stake_credential = StakeCredentialActiveModel {
            credential: Set(credential),
            ..Default::default()
        };

        let stake_credential = stake_credential.insert(txn).await?;

        let tx_credential = TxCredentialActiveModel {
            credential_id: Set(stake_credential.id),
            tx_id: Set(tx.id),
            relation: Set(relation),
            ..Default::default()
        };

        tx_credential.save(txn).await?;

        Ok(stake_credential)
    }
}

fn block_with_era(era: Era, payload: &[u8]) -> anyhow::Result<(MultiEraBlock, i32)> {
    let data = match era {
        Era::Byron => {
            let block = byron::Block::decode_fragment(payload)
                .map_err(|_| anyhow!("failed to decode cbor"))?;

            (MultiEraBlock::Byron(Box::new(block)), 0)
        }
        rest => {
            let alonzo::BlockWrapper(_, block) = alonzo::BlockWrapper::decode_fragment(payload)
                .map_err(|_| anyhow!("failed to decode cbor"))?;

            let box_block = Box::new(block);

            match rest {
                Era::Shelley => (MultiEraBlock::Compatible(box_block), 1),
                Era::Allegra => (MultiEraBlock::Compatible(box_block), 2),
                Era::Mary => (MultiEraBlock::Compatible(box_block), 3),
                Era::Alonzo => (MultiEraBlock::Compatible(box_block), 4),
                _ => unreachable!(),
            }
        }
    };

    Ok(data)
}
