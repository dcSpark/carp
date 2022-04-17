use crate::perf_aggregator::PerfAggregator;
use crate::relation_map::RelationMap;
use cryptoxide::blake2b::Blake2b;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::{
    byron::{self, TxIn, TxOut},
    Fragment,
};
use std::ops::Deref;

pub async fn process_byron_block(
    perf_aggregator: &mut PerfAggregator,
    time_counter: &mut std::time::Instant,
    txn: &DatabaseTransaction,
    db_block: &BlockModel,
    byron_block: &byron::Block,
) -> Result<(), DbErr> {
    match byron_block {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => (),
        byron::Block::MainBlock(main_block) => {
            for (idx, tx_body) in main_block.body.tx_payload.iter().enumerate() {
                let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                let tx_payload = tx_body.encode_fragment().unwrap();

                let transaction = TransactionActiveModel {
                    hash: Set(tx_hash.to_vec()),
                    block_id: Set(db_block.id),
                    tx_index: Set(idx as i32),
                    payload: Set(tx_payload),
                    is_valid: Set(true), // always true in Byron
                    ..Default::default()
                };

                let transaction = transaction.insert(txn).await?;

                // unused for Byron
                let mut vkey_relation_map = RelationMap::default();

                perf_aggregator.transaction_insert += time_counter.elapsed();
                *time_counter = std::time::Instant::now();

                for (idx, output) in tx_body.transaction.outputs.iter().enumerate() {
                    insert_byron_output(txn, &transaction, output, idx).await?;
                }

                perf_aggregator.transaction_output_insert += time_counter.elapsed();
                *time_counter = std::time::Instant::now();

                for (idx, input) in tx_body.transaction.inputs.iter().enumerate() {
                    insert_byron_input(&mut vkey_relation_map, txn, &transaction, input, idx)
                        .await?;
                }

                perf_aggregator.transaction_input_insert += time_counter.elapsed();
                *time_counter = std::time::Instant::now();
            }
        }
    }

    Ok(())
}

async fn insert_byron_output(
    txn: &DatabaseTransaction,
    transaction: &TransactionModel,
    output: &TxOut,
    idx: usize,
) -> Result<(), DbErr> {
    let mut address_payload = output.address.encode_fragment().unwrap();

    let address = crate::era_common::insert_address(&mut address_payload, txn).await?;

    let tx_output = TransactionOutputActiveModel {
        payload: Set(output.encode_fragment().unwrap()),
        address_id: Set(address.id),
        tx_id: Set(transaction.id),
        output_index: Set(idx as i32),
        ..Default::default()
    };

    tx_output.save(txn).await?;

    Ok(())
}

async fn insert_byron_input(
    vkey_relation_map: &mut RelationMap,
    txn: &DatabaseTransaction,
    transaction: &TransactionModel,
    input: &TxIn,
    idx: usize,
) -> Result<(), DbErr> {
    let (tx_hash, index) = match input {
        TxIn::Variant0(wrapped) => wrapped.deref(),
        TxIn::Other(index, tx_hash) => {
            // Note: Oura uses "other" to future proof itself against changes in the binary spec
            todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
        }
    };

    crate::era_common::insert_input(
        vkey_relation_map,
        transaction.id,
        idx as i32,
        *index as u64,
        tx_hash,
        txn,
    )
    .await?;

    Ok(())
}

fn blake2b256(data: &[u8]) -> [u8; 32] {
    let mut out = [0; 32];
    Blake2b::blake2b(&mut out, data, &[]);
    out
}
