use std::ops::Deref;

use anyhow::anyhow;
use oura::{
    model::{Era, EventData},
    pipelining::StageReceiver,
};
use pallas::ledger::primitives::{
    alonzo::{
        self, crypto, Certificate, StakeCredential, TransactionBody, TransactionBodyComponent,
    },
    byron, Fragment,
};

use entity::{
    prelude::{
        AddressActiveModel, Block, BlockActiveModel, BlockColumn, StakeCredentialActiveModel,
        TransactionActiveModel, TransactionModel, TransactionOutputActiveModel,
        TxCredentialActiveModel,
    },
    sea_orm::{prelude::*, DatabaseTransaction, Set, TransactionTrait},
};
use migration::DbErr;

#[derive(Debug)]
enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}

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
                    let hash = hex::decode(&block_record.hash)?;
                    let payload = hex::decode(block_record.cbor_hex.as_ref().unwrap())?;

                    let (multi_block, era) = block_with_era(block_record.era, &payload)?;

                    self.conn
                        .transaction::<_, (), DbErr>(|txn| {
                            Box::pin(async move {
                                let block = BlockActiveModel {
                                    era: Set(era),
                                    hash: Set(hash),
                                    height: Set(block_record.number as i64),
                                    epoch: Set(0),
                                    slot: Set(block_record.slot as i64),
                                    payload: Set(payload),
                                    ..Default::default()
                                };

                                let block = block.insert(txn).await?;

                                match multi_block {
                                    MultiEraBlock::Byron(byron_block) => {
                                        match byron_block.deref() {
                                            byron::Block::MainBlock(_main_block) => {}
                                            byron::Block::EbBlock(_main_block) => {}
                                        }
                                    }
                                    MultiEraBlock::Compatible(alonzo_block) => {
                                        for (idx, tx_body) in alonzo_block
                                            .deref()
                                            .transaction_bodies
                                            .iter()
                                            .enumerate()
                                        {
                                            let tx_hash =
                                                crypto::hash_transaction(tx_body).to_vec();

                                            let payload = TransactionBody::encode_fragment(tx_body)
                                                .expect("should not fail");

                                            let transaction = TransactionActiveModel {
                                                hash: Set(tx_hash),
                                                block_id: Set(block.id),
                                                tx_index: Set(idx as i32),
                                                payload: Set(payload),
                                                is_valid: Set(false),
                                                ..Default::default()
                                            };

                                            let transaction = transaction.insert(txn).await?;

                                            for component in tx_body.iter() {
                                                insert_certificates(&transaction, component, txn)
                                                    .await?;
                                            }

                                            for component in tx_body.iter() {
                                                match component {
                                                    TransactionBodyComponent::Inputs(inputs) => {
                                                        tracing::debug!("INPUTS: {}", inputs.len());
                                                    }
                                                    TransactionBodyComponent::Outputs(outputs) => {
                                                        for (idx, output) in
                                                            outputs.iter().enumerate()
                                                        {
                                                            let address = AddressActiveModel {
                                                                payload: Set(output
                                                                    .address
                                                                    .to_vec()),
                                                                ..Default::default()
                                                            };

                                                            let address =
                                                                address.insert(txn).await?;

                                                            let tx_output =
                                                                TransactionOutputActiveModel {
                                                                    payload: Set(vec![]),
                                                                    address_id: Set(address.id),
                                                                    tx_id: Set(transaction.id),
                                                                    output_index: Set(idx as i32),
                                                                    ..Default::default()
                                                                };

                                                            tx_output.save(txn).await?;
                                                        }
                                                        tracing::debug!(
                                                            "OUTPUTS: {}",
                                                            outputs.len()
                                                        );
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                    }
                                }

                                Ok(())
                            })
                        })
                        .await?;

                    tracing::info!(
                        "inserted block    - slot: {}, hash: {}",
                        block_record.slot,
                        block_record.hash
                    );
                }
                EventData::RollBack {
                    block_hash,
                    block_slot,
                } => {
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gte(block_slot))
                        .exec(self.conn)
                        .await?;

                    tracing::info!(
                        "rollback complete - slot: {}, hash: {}",
                        block_slot,
                        block_hash
                    );
                }
                _ => (),
            }
        }
    }
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
                    insert_credential(tx, credential, txn).await?;
                }
                Certificate::StakeRegistration(credential) => {
                    insert_credential(tx, credential, txn).await?;
                }
                Certificate::StakeDeregistration(credential) => {
                    insert_credential(tx, credential, txn).await?;
                }
                _ => (),
            }
        }
    }

    Ok(())
}

async fn insert_credential(
    tx: &TransactionModel,
    credential: &StakeCredential,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let credential = credential.encode_fragment().expect("should not fail");

    let stake_credential = StakeCredentialActiveModel {
        credential: Set(credential),
        ..Default::default()
    };

    let stake_credential = stake_credential.insert(txn).await?;

    let tx_credential = TxCredentialActiveModel {
        credential_id: Set(stake_credential.id),
        tx_id: Set(tx.id),
        // TODO: what should this be?
        relation: Set(0),
        ..Default::default()
    };

    tx_credential.save(txn).await?;

    Ok(())
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
