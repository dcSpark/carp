use std::ops::Deref;

use anyhow::anyhow;
use cardano_serialization_lib::address::{
    BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use oura::{
    model::{BlockRecord, Era, EventData},
    pipelining::StageReceiver,
};
use pallas::{
    crypto::hash::{Hash, Hasher},
    ledger::primitives::{
        alonzo::{self, Certificate, TransactionBodyComponent},
        byron::{self, TxIn},
        Fragment,
    },
};

use crate::types::{AddressCredentialRelation, MultiEraBlock, TxCredentialRelation};
use entity::{
    prelude::*,
    sea_orm::{
        prelude::*, ColumnTrait, DatabaseTransaction, JoinType, QuerySelect, Set, TransactionTrait,
    },
};
use migration::DbErr;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

impl<'a> Config<'a> {
    pub async fn bootstrap(&self, input: StageReceiver) -> anyhow::Result<()> {
        loop {
            let event = input.recv()?;

            let data = event.data.clone();

            match data {
                EventData::Block(block_record) => {
                    self.conn
                        .transaction::<_, (), DbErr>(|txn| Box::pin(insert(block_record, txn)))
                        .await?;
                }
                EventData::RollBack { block_slot, .. } => {
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gt(block_slot))
                        .exec(self.conn)
                        .await?;
                }
                _ => (),
            }
        }
    }
}

async fn insert(block_record: BlockRecord, txn: &DatabaseTransaction) -> Result<(), DbErr> {
    let hash = hex::decode(&block_record.hash).unwrap();
    let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();

    let (multi_block, era) = block_with_era(block_record.era, &block_payload).unwrap();

    let block = BlockActiveModel {
        era: Set(era),
        hash: Set(hash),
        height: Set(block_record.number as i64),
        epoch: Set(0),
        slot: Set(block_record.slot as i64),
        payload: Set(block_payload),
        ..Default::default()
    };

    let block = block.insert(txn).await?;

    match multi_block {
        MultiEraBlock::Byron(byron_block) => match byron_block.deref() {
            byron::Block::EbBlock(_) => (),
            byron::Block::MainBlock(main_block) => {
                for (idx, tx_body) in main_block.body.tx_payload.iter().enumerate() {
                    let tx_hash = Hasher::<256>::hash_cbor(tx_body).to_vec();

                    let tx_payload = tx_body.encode_fragment().unwrap();

                    let transaction = TransactionActiveModel {
                        hash: Set(tx_hash),
                        block_id: Set(block.id),
                        tx_index: Set(idx as i32),
                        payload: Set(tx_payload),
                        is_valid: Set(true),
                        ..Default::default()
                    };

                    let transaction = transaction.insert(txn).await?;

                    for (idx, input) in tx_body.transaction.inputs.iter().enumerate() {
                        let (tx_hash, index) = match input {
                            TxIn::Variant0(wrapped) => wrapped.deref(),
                            TxIn::Other(index, tx_hash) => {
                                todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
                            }
                        };

                        insert_input(&transaction, idx as i32, *index as u64, tx_hash, txn).await?;
                    }

                    for (idx, output) in tx_body.transaction.outputs.iter().enumerate() {
                        let address_payload = output.address.encode_fragment().unwrap();

                        let address = insert_address(address_payload, txn).await?;

                        let tx_output = TransactionOutputActiveModel {
                            payload: Set(output.encode_fragment().unwrap()),
                            address_id: Set(address.id),
                            tx_id: Set(transaction.id),
                            output_index: Set(idx as i32),
                            ..Default::default()
                        };

                        tx_output.save(txn).await?;
                    }
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
                let body =
                    &cardano_serialization_lib::TransactionBody::from_bytes(body_payload).unwrap();

                let witness_set_payload = tx_witness_set.encode_fragment().unwrap();
                let witness_set = &cardano_serialization_lib::TransactionWitnessSet::from_bytes(
                    witness_set_payload,
                )
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
                    .unwrap()
                });

                let mut temp_tx =
                    cardano_serialization_lib::Transaction::new(body, witness_set, auxiliary_data);

                let mut is_valid = true;

                if let Some(ref invalid_txs) = alonzo_block.invalid_transactions {
                    is_valid = invalid_txs.iter().any(|i| *i as usize == idx)
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

                for component in tx_body.iter() {
                    insert_certificates(&transaction, component, txn).await?;
                }

                for component in tx_body.iter() {
                    match component {
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
                        }
                        TransactionBodyComponent::Outputs(outputs) => {
                            for (idx, output) in outputs.iter().enumerate() {
                                use cardano_serialization_lib::address::Address;

                                let address = insert_address(output.address.to_vec(), txn).await?;

                                let addr = Address::from_bytes(output.address.to_vec()).unwrap();

                                let tx_relation = TxCredentialRelation::Output;
                                let address_relation = AddressCredentialRelation::PaymentKey;

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

                                    let address_relation = AddressCredentialRelation::StakeKey;

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
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    Ok(())
}

async fn insert_address(
    payload: Vec<u8>,
    txn: &DatabaseTransaction,
) -> Result<AddressModel, DbErr> {
    let addr = Address::find()
        .filter(AddressColumn::Payload.eq(payload.clone()))
        .one(txn)
        .await?;

    if let Some(addr) = addr {
        Ok(addr)
    } else {
        let address = AddressActiveModel {
            payload: Set(payload),
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
    let tx_output = TransactionOutput::find()
        .filter(TransactionOutputColumn::OutputIndex.eq(index))
        .join(
            JoinType::LeftJoin,
            TransactionOutputRelation::Transaction.def(),
        )
        .filter(TransactionColumn::Hash.eq(tx_hash.to_vec()))
        .one(txn)
        .await?;

    let tx_output = tx_output.unwrap();

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
                    (credential, TxCredentialRelation::StakeDelegation)
                }
                Certificate::StakeRegistration(credential) => {
                    (credential, TxCredentialRelation::StakeRegistration)
                }
                Certificate::StakeDeregistration(credential) => {
                    (credential, TxCredentialRelation::StakeDeregistration)
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
