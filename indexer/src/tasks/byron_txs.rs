extern crate shred;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::{byron, Fragment};
use shred::{ResourceId, System, SystemData, World, Write, WriteExpect};

use crate::byron::blake2b256;

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: WriteExpect<'a, Vec<TransactionModel>>,
}

pub struct ByronTransactionTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: (&'a byron::Block, &'a BlockModel),
    pub handle: &'a tokio::runtime::Handle,
}

impl<'a> System<'a> for ByronTransactionTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        println!("{}", "foo");
        let result = self
            .handle
            .block_on(handle_tx(self.db_tx, self.block))
            .unwrap();
        *bundle.byron_txs = result;
    }
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: (&byron::Block, &BlockModel),
) -> Result<Vec<TransactionModel>, DbErr> {
    println!("{}", "asdf");
    match &block.0 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            if main_block.body.tx_payload.is_empty() {
                return Ok(vec![]);
            }

            let transaction_inserts =
                Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
                    |(idx, tx_body)| {
                        let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                        let tx_payload = tx_body.encode_fragment().unwrap();

                        TransactionActiveModel {
                            hash: Set(tx_hash.to_vec()),
                            block_id: Set(block.1.id),
                            tx_index: Set(idx as i32),
                            payload: Set(tx_payload),
                            is_valid: Set(true), // always true in Byron
                            ..Default::default()
                        }
                    },
                ))
                .exec_many_with_returning(db_tx)
                .await?;
            Ok(transaction_inserts)
        }
    }
}
