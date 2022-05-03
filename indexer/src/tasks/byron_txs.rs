extern crate shred;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::{byron, Fragment};
use shred::{DispatcherBuilder, Read, ReadExpect, ResourceId, System, SystemData, World, Write};
use std::{thread, time};

use crate::byron::blake2b256;

#[derive(SystemData)]
pub struct Data<'a> {
    dbTx: ReadExpect<'a, DatabaseTransaction>,
    block: ReadExpect<'a, (BlockModel, byron::Block)>,
}

pub struct ByronTransactionTask;

impl<'a> System<'a> for ByronTransactionTask {
    type SystemData = Data<'a>;

    async fn run(&mut self, bundle: Data<'a>) {
        match &bundle.block.1 {
            // Byron era had Epoch-boundary blocks for calculating stake distribution changes
            // they don't contain any txs, so we can just ignore them
            byron::Block::EbBlock(_) => (),
            byron::Block::MainBlock(main_block) => {
                if main_block.body.tx_payload.is_empty() {
                    return;
                }
                let transaction_inserts =
                    Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
                        |(idx, tx_body)| {
                            let tx_hash =
                                blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                            let tx_payload = tx_body.encode_fragment().unwrap();

                            TransactionActiveModel {
                                hash: Set(tx_hash.to_vec()),
                                block_id: Set(bundle.block.0.id),
                                tx_index: Set(idx as i32),
                                payload: Set(tx_payload),
                                is_valid: Set(true), // always true in Byron
                                ..Default::default()
                            }
                        },
                    ))
                    .exec_many_with_returning(&bundle.dbTx.into())
                    .await?;
            }
        }
    }
}
