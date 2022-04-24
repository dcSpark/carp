use crate::perf_aggregator::PerfAggregator;
use crate::relation_map::RelationMap;
use cryptoxide::blake2b::Blake2b;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::{
    codec::utils::MaybeIndefArray,
    ledger::primitives::{
        byron::{self, TxIn, TxOut},
        Fragment,
    },
};

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
            if main_block.body.tx_payload.is_empty() {
                return Ok(());
            }
            let transaction_inserts =
                Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
                    |(idx, tx_body)| {
                        let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                        let tx_payload = tx_body.encode_fragment().unwrap();

                        TransactionActiveModel {
                            hash: Set(tx_hash.to_vec()),
                            block_id: Set(db_block.id),
                            tx_index: Set(idx as i32),
                            payload: Set(tx_payload),
                            is_valid: Set(true), // always true in Byron
                            ..Default::default()
                        }
                    },
                ))
                .exec_many_with_returning(txn)
                .await?;

            perf_aggregator.transaction_insert += time_counter.elapsed();
            *time_counter = std::time::Instant::now();

            let tx_outputs: Vec<_> = main_block
                .body
                .tx_payload
                .iter()
                .map(|payload| &payload.transaction.outputs)
                .zip(&transaction_inserts)
                .collect();

            // note: outputs have to be added before inputs
            insert_byron_outputs(txn, &tx_outputs).await?;

            perf_aggregator.transaction_output_insert += time_counter.elapsed();
            *time_counter = std::time::Instant::now();

            // unused for Byron
            let mut vkey_relation_map = RelationMap::default();

            let all_inputs: Vec<(
                Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
                i64,
            )> = main_block
                .body
                .tx_payload
                .iter()
                .zip(&transaction_inserts)
                .map(|(tx_payload, cardano_tx_in_db)| {
                    let inputs: Vec<pallas::ledger::primitives::alonzo::TransactionInput> =
                        tx_payload
                            .transaction
                            .inputs
                            .iter()
                            .map(|input| byron_input_to_alonzo(&input))
                            .collect();
                    (inputs, cardano_tx_in_db.id)
                })
                .collect();
            crate::era_common::insert_inputs(
                &mut vkey_relation_map,
                &all_inputs
                    .iter()
                    .map(|inputs| (&inputs.0, inputs.1))
                    .collect(),
                txn,
            )
            .await?;

            perf_aggregator.transaction_input_insert += time_counter.elapsed();
            *time_counter = std::time::Instant::now();
        }
    }

    Ok(())
}

async fn insert_byron_outputs(
    txn: &DatabaseTransaction,
    outputs: &Vec<(&MaybeIndefArray<TxOut>, &TransactionModel)>,
) -> Result<(), DbErr> {
    let (_, address_map) = crate::era_common::insert_addresses(
        &outputs
            .iter()
            .flat_map(|pair| pair.0.iter())
            .map(|output| output.address.encode_fragment().unwrap())
            .collect(),
        txn,
    )
    .await?;

    TransactionOutput::insert_many(
        outputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(
                |((output_index, output), tx_id)| TransactionOutputActiveModel {
                    payload: Set(output.encode_fragment().unwrap()),
                    address_id: Set(address_map
                        .get(&output.address.encode_fragment().unwrap())
                        .unwrap()
                        .id),
                    tx_id: Set(tx_id.id),
                    output_index: Set(output_index as i32),
                    ..Default::default()
                },
            ),
    )
    .exec(txn)
    .await?;

    Ok(())
}

fn byron_input_to_alonzo(input: &TxIn) -> pallas::ledger::primitives::alonzo::TransactionInput {
    match input {
        TxIn::Variant0(wrapped) => pallas::ledger::primitives::alonzo::TransactionInput {
            transaction_id: wrapped.0 .0.clone(),
            index: wrapped.0 .1 as u64,
        },
        TxIn::Other(index, tx_hash) => {
            // Note: Oura uses "other" to future proof itself against changes in the binary spec
            todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
        }
    }
}

fn blake2b256(data: &[u8]) -> [u8; 32] {
    let mut out = [0; 32];
    Blake2b::blake2b(&mut out, data, &[]);
    out
}
