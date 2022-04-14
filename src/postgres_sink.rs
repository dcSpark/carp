use std::ops::Deref;

use anyhow::anyhow;
use cardano_multiplatform_lib::address::{
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
        prelude::*, sea_query::OnConflict, ColumnTrait, DatabaseTransaction, JoinType, QuerySelect,
        Set, TransactionTrait,
    },
};
use migration::DbErr;
use std::time::Duration;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct PerfAggregator {
    block_parse: Duration,
    block_insertion: Duration,
    transaction_insert: Duration,
    transaction_input_insert: Duration,
    transaction_output_insert: Duration,
    certificate_insert: Duration,
    collateral_insert: Duration,
    withdrawal_insert: Duration,
    required_signer_insert: Duration,
    block_fetch: Duration,
    rollback: Duration,
    overhead: Duration,
}
impl PerfAggregator {
    pub fn new() -> Self {
        Self {
            block_parse: Duration::new(0, 0),
            block_insertion: Duration::new(0, 0),
            transaction_insert: Duration::new(0, 0),
            transaction_input_insert: Duration::new(0, 0),
            transaction_output_insert: Duration::new(0, 0),
            certificate_insert: Duration::new(0, 0),
            collateral_insert: Duration::new(0, 0),
            withdrawal_insert: Duration::new(0, 0),
            required_signer_insert: Duration::new(0, 0),
            block_fetch: Duration::new(0, 0),
            rollback: Duration::new(0, 0),
            overhead: Duration::new(0, 0),
        }
    }
    pub fn set_overhead(&mut self, total_duration: &Duration) {
        let non_duration_sum = self.block_parse
            + self.block_insertion
            + self.transaction_insert
            + self.transaction_input_insert
            + self.transaction_output_insert
            + self.certificate_insert
            + self.collateral_insert
            + self.withdrawal_insert
            + self.required_signer_insert
            + self.block_fetch
            + self.rollback;
        self.overhead = *total_duration - non_duration_sum
    }
}
impl std::ops::Add for PerfAggregator {
    type Output = PerfAggregator;

    fn add(self, other: Self) -> Self {
        Self {
            block_parse: self.block_parse + other.block_parse,
            block_insertion: self.block_insertion + other.block_insertion,
            transaction_insert: self.transaction_insert + other.transaction_insert,
            transaction_input_insert: self.transaction_input_insert
                + other.transaction_input_insert,
            transaction_output_insert: self.transaction_output_insert
                + other.transaction_output_insert,
            certificate_insert: self.certificate_insert + other.certificate_insert,
            collateral_insert: self.collateral_insert + other.collateral_insert,
            withdrawal_insert: self.withdrawal_insert + other.withdrawal_insert,
            required_signer_insert: self.required_signer_insert + other.required_signer_insert,
            block_fetch: self.block_fetch + other.block_fetch,
            rollback: self.rollback + other.rollback,
            overhead: self.overhead + other.overhead,
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
            let event_fetch_start = std::time::Instant::now();
            let event = input.recv()?;
            perf_aggregator.block_fetch += event_fetch_start.elapsed();

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
                    match block_slot {
                        0 => tracing::info!("Rolling back to genesis"),
                        _ => tracing::info!("Rolling back to slot {}", block_slot - 1),
                    };
                    let rollback_start = std::time::Instant::now();
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gt(block_slot))
                        .exec(self.conn)
                        .await?;
                    perf_aggregator.rollback += rollback_start.elapsed();
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
    let mut perf_aggregator = PerfAggregator::new();
    let mut time_counter = std::time::Instant::now();

    let hash = hex::decode(&block_record.hash).unwrap();
    let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();

    perf_aggregator.block_parse += time_counter.elapsed();
    time_counter = std::time::Instant::now();

    let (multi_block, era) = block_with_era(block_record.era, &block_payload).unwrap();

    let block = BlockActiveModel {
        era: Set(era),
        hash: Set(hash),
        height: Set(block_record.number as i32),
        epoch: Set(0),
        slot: Set(block_record.slot as i32),
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
                        is_valid: Set(true), // always true in Byron
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
                            output_index: Set(idx as i32),
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
                let body = &cardano_multiplatform_lib::TransactionBody::from_bytes(body_payload)
                    .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                    .unwrap();

                let witness_set_payload = tx_witness_set.encode_fragment().unwrap();
                let witness_set = &cardano_multiplatform_lib::TransactionWitnessSet::from_bytes(
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

                    cardano_multiplatform_lib::metadata::AuxiliaryData::from_bytes(
                        auxiliary_data_payload,
                    )
                    .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
                    .unwrap()
                });

                let mut temp_tx =
                    cardano_multiplatform_lib::Transaction::new(body, witness_set, auxiliary_data);

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
                                use cardano_multiplatform_lib::address::Address;

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
                                    output_index: Set(idx as i32),
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
                            // note: we consider collateral as just another kind of input instead of a separate table
                            // you can use the is_valid field to know what kind of input it actually is
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
                        TransactionBodyComponent::Withdrawals(withdrawal_pairs) => {
                            for pair in withdrawal_pairs.deref() {
                                let credential = pair.0.encode_fragment().unwrap();
                                insert_credential(
                                    &transaction,
                                    credential,
                                    txn,
                                    TxCredentialRelationValue::Withdrawal.into(),
                                )
                                .await?;
                            }
                            perf_aggregator.withdrawal_insert += time_counter.elapsed();
                            time_counter = std::time::Instant::now();
                        }
                        TransactionBodyComponent::RequiredSigners(key_hashes) => {
                            for &signer in key_hashes.iter() {
                                let owner_credential = pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(signer).encode_fragment().unwrap();
                                insert_credential(
                                    &transaction,
                                    owner_credential,
                                    txn,
                                    TxCredentialRelationValue::RequiredSigner.into(),
                                )
                                .await?;
                            }
                            perf_aggregator.required_signer_insert += time_counter.elapsed();
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
    tx_relation: TxCredentialRelationValue,
    address_relation: i32,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let stake_credential = insert_credential(tx, payload, txn, tx_relation).await?;

    let address_credential = AddressCredentialActiveModel {
        credential_id: Set(stake_credential.id),
        address_id: Set(address.id),
        relation: Set(address_relation),
    };

    let on_conflict = OnConflict::columns([
        AddressCredentialColumn::AddressId,
        AddressCredentialColumn::CredentialId,
        AddressCredentialColumn::Relation,
    ])
    .do_nothing()
    .to_owned();
    address_credential.insert_or(txn, &on_conflict).await?;

    Ok(())
}

async fn insert_tx_credential(
    stake_credential: &StakeCredentialModel,
    tx: &TransactionModel,
    relation: TxCredentialRelationValue,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let tx_credential = TxCredentialActiveModel {
        credential_id: Set(stake_credential.id),
        tx_id: Set(tx.id),
        relation: Set(relation.into()),
    };

    let on_conflict = OnConflict::columns([
        TxCredentialColumn::CredentialId,
        TxCredentialColumn::TxId,
        TxCredentialColumn::Relation,
    ])
    .do_nothing()
    .to_owned();
    tx_credential.insert_or(txn, &on_conflict).await?;

    Ok(())
}

async fn insert_input(
    tx: &TransactionModel,
    index_in_input: i32,
    index_in_output: u64,
    tx_hash: &Hash<32>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    // 1) Get the UTXO this input is spending
    let tx_output = TransactionOutput::find()
        .inner_join(Transaction)
        .filter(TransactionOutputColumn::OutputIndex.eq(index_in_output))
        .filter(TransactionColumn::Hash.eq(tx_hash.to_vec()))
        .one(txn)
        .await?;

    let tx_output = tx_output.unwrap();

    let is_byron = match cardano_multiplatform_lib::TransactionOutput::from_bytes(tx_output.payload)
    {
        Ok(parsed_output) => parsed_output.address().as_byron().is_some(),
        // TODO: remove this once we've parsed the genesis block correctly instead of inserting dummy data
        Err(_) => true,
    };
    // Byron addresses don't contain stake credentials, so we can skip them
    if !is_byron {
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

        // 3) Associate the stake credentials to this transaction
        let relation = TxCredentialRelationValue::Input;
        for stake_credential in stake_credentials {
            insert_tx_credential(&stake_credential, &tx, relation, txn).await?;
        }
    }

    // 4) Add input itself
    let tx_input = TransactionInputActiveModel {
        utxo_id: Set(tx_output.id),
        tx_id: Set(tx.id),
        input_index: Set(index_in_input),
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
            match cert {
                Certificate::StakeDelegation(credential, _) => {
                    let credential = credential.encode_fragment().unwrap();
                    insert_credential(tx, credential, txn, TxCredentialRelationValue::StakeDelegation.into()).await?;
                    // TODO: Add delegation target as well? Probably should hide behind a flag for performance
                }
                Certificate::StakeRegistration(credential) => {
                    let credential = credential.encode_fragment().unwrap();
                    insert_credential(tx, credential, txn, TxCredentialRelationValue::StakeRegistration.into()).await?;
                }
                Certificate::StakeDeregistration(credential) => {
                    let credential = credential.encode_fragment().unwrap();
                    insert_credential(tx, credential, txn, TxCredentialRelationValue::StakeDeregistration.into()).await?;
                }
                Certificate::PoolRegistration { operator, pool_owners, reward_account, .. } => {
                    let operator_credential = pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(operator.clone())
                    .encode_fragment().unwrap();
                    insert_credential(tx, operator_credential, txn, TxCredentialRelationValue::PoolOperator.into()).await?;

                    let reward_addr = RewardAddress::from_address(&cardano_multiplatform_lib::address::Address::from_bytes(reward_account.to_vec()).unwrap()).unwrap();
                    let reward_key_hash: [u8; 28] = reward_addr.payment_cred().to_keyhash().unwrap().to_bytes().try_into().unwrap();
                    match &reward_addr.payment_cred().kind() {
                        cardano_multiplatform_lib::address::StakeCredKind::Key => {
                            let reward_credential = pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(Hash::<28>::from(reward_key_hash)).encode_fragment().unwrap();
                            insert_credential(tx, reward_credential, txn, TxCredentialRelationValue::PoolReward.into()).await?;
                        },
                        _ => {},
                    };

                    for &owner in pool_owners.iter() {
                        let owner_credential = pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(owner).encode_fragment().unwrap();
                        insert_credential(tx, owner_credential, txn, TxCredentialRelationValue::PoolOperator.into()).await?;
                    };
                }
                Certificate::PoolRetirement(key_hash, _) => {
                    let operator_credential = pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(key_hash.clone()).encode_fragment().unwrap();
                    insert_credential(tx, operator_credential, txn, TxCredentialRelationValue::PoolOperator.into()).await?;
                }
                Certificate::GenesisKeyDelegation(_, _, _) => {
                    // genesis keys aren't stake credentials
                }
                Certificate::MoveInstantaneousRewardsCert(mir) => {
                    match &mir.target {
                        pallas::ledger::primitives::alonzo::InstantaneousRewardTarget::StakeCredentials(credential_pairs) => {
                            for pair in credential_pairs.deref() {
                                let credential = pair.0.encode_fragment().unwrap();
                                insert_credential(tx, credential, txn, TxCredentialRelationValue::MirRecipient.into()).await?;
                            }
                        },
                        _ => {},
                    }
                }
            };
        }
    }

    Ok(())
}

async fn insert_credential(
    tx: &TransactionModel,
    credential: Vec<u8>,
    txn: &DatabaseTransaction,
    relation: TxCredentialRelationValue,
) -> Result<StakeCredentialModel, DbErr> {
    let sc = StakeCredential::find()
        .filter(StakeCredentialColumn::Credential.eq(credential.clone()))
        .one(txn)
        .await?;

    if let Some(stake_credential) = sc {
        insert_tx_credential(&stake_credential, &tx, relation, txn).await?;

        Ok(stake_credential)
    } else {
        let stake_credential = StakeCredentialActiveModel {
            credential: Set(credential),
            ..Default::default()
        };

        let stake_credential = stake_credential.insert(txn).await?;

        insert_tx_credential(&stake_credential, &tx, relation, txn).await?;

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
