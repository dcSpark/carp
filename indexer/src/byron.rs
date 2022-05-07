use std::collections::BTreeMap;

use crate::tasks::byron_outputs::ByronOutputTask;
use crate::tasks::byron_txs::ByronTransactionTask;
use crate::{era_common::get_truncated_address, perf_aggregator::PerfAggregator};
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
use shred::{DispatcherBuilder, World};
use tokio::runtime::Runtime;

pub async fn process_byron_block(
    perf_aggregator: &mut PerfAggregator,
    time_counter: &mut std::time::Instant,
    txn: &DatabaseTransaction,
    block: (&byron::Block, &BlockModel),
) -> Result<(), DbErr> {
    println!("{}", "enter");
    let rt = Runtime::new().unwrap();

    let world = World::empty();
    let mut dispatcher = DispatcherBuilder::new()
        .with(
            ByronTransactionTask {
                db_tx: txn,
                block,
                handle: rt.handle(),
            },
            "ByronTransactionTask",
            &[],
        )
        .with(
            ByronOutputTask {
                db_tx: txn,
                block,
                handle: rt.handle(),
            },
            "ByronOutputTask",
            &["ByronTransactionTask"],
        )
        .build();
    dispatcher.dispatch(&world);
    println!("{}", "done");
    // match byron_block {
    //     // Byron era had Epoch-boundary blocks for calculating stake distribution changes
    //     // they don't contain any txs, so we can just ignore them
    //     byron::Block::EbBlock(_) => (),
    //     byron::Block::MainBlock(main_block) => {
    //         if main_block.body.tx_payload.is_empty() {
    //             return Ok(());
    //         }
    //         let transaction_inserts =
    //             Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
    //                 |(idx, tx_body)| {
    //                     let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

    //                     let tx_payload = tx_body.encode_fragment().unwrap();

    //                     TransactionActiveModel {
    //                         hash: Set(tx_hash.to_vec()),
    //                         block_id: Set(db_block.id),
    //                         tx_index: Set(idx as i32),
    //                         payload: Set(tx_payload),
    //                         is_valid: Set(true), // always true in Byron
    //                         ..Default::default()
    //                     }
    //                 },
    //             ))
    //             .exec_many_with_returning(txn)
    //             .await?;

    //         perf_aggregator.transaction_insert += time_counter.elapsed();
    //         *time_counter = std::time::Instant::now();

    //         let tx_outputs: Vec<_> = main_block
    //             .body
    //             .tx_payload
    //             .iter()
    //             .map(|payload| &payload.transaction.outputs)
    //             .zip(&transaction_inserts)
    //             .collect();

    //         // insert addresses
    //         let (_, address_map) = crate::era_common::insert_addresses(
    //             &tx_outputs
    //                 .iter()
    //                 .flat_map(|pair| pair.0.iter())
    //                 .map(|output| output.address.encode_fragment().unwrap())
    //                 .collect(),
    //             txn,
    //         )
    //         .await?;
    //         perf_aggregator.addr_insert += time_counter.elapsed();
    //         *time_counter = std::time::Instant::now();

    //         // note: outputs have to be added before inputs
    //         insert_byron_outputs(txn, &address_map, &tx_outputs).await?;
    //         perf_aggregator.transaction_output_insert += time_counter.elapsed();
    //         *time_counter = std::time::Instant::now();

    //         let all_inputs: Vec<(
    //             Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    //             i64,
    //         )> = main_block
    //             .body
    //             .tx_payload
    //             .iter()
    //             .zip(&transaction_inserts)
    //             .map(|(tx_payload, cardano_tx_in_db)| {
    //                 let inputs: Vec<pallas::ledger::primitives::alonzo::TransactionInput> =
    //                     tx_payload
    //                         .transaction
    //                         .inputs
    //                         .iter()
    //                         .map(byron_input_to_alonzo)
    //                         .collect();
    //                 (inputs, cardano_tx_in_db.id)
    //             })
    //             .collect();

    //         let flattened_inputs: Vec<(
    //             &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    //             i64,
    //         )> = all_inputs
    //             .iter()
    //             .map(|inputs| (&inputs.0, inputs.1))
    //             .collect();
    //         let outputs_for_inputs =
    //             crate::era_common::get_outputs_for_inputs(&flattened_inputs, txn).await?;

    //         let input_to_output_map =
    //             crate::era_common::gen_input_to_output_map(&outputs_for_inputs);
    //         crate::era_common::insert_inputs(&flattened_inputs, &input_to_output_map, txn).await?;

    //         perf_aggregator.transaction_input_insert += time_counter.elapsed();
    //         *time_counter = std::time::Instant::now();
    //     }
    // }

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

pub fn blake2b256(data: &[u8]) -> [u8; 32] {
    let mut out = [0; 32];
    Blake2b::blake2b(&mut out, data, &[]);
    out
}
