use crate::{
    dsl::default_impl::has_transaction_byron, dsl::task_macro::*,
    era_common::transactions_from_hashes, utils::blake2b256,
};
use entity::sea_orm::Set;
use pallas::ledger::primitives::{byron, Fragment};

use super::byron_block::ByronBlockTask;
use crate::config::ReadonlyConfig::ReadonlyConfig;

carp_task! {
  name ByronTransactionTask;
  configuration ReadonlyConfig;
  doc "Adds the transactions in the block to the database";
  era byron;
  dependencies [ByronBlockTask];
  read [byron_block];
  write [byron_txs];
  should_add_task |block, _properties| {
    has_transaction_byron(block.1)
  };
  execute |previous_data, task| handle_tx(
      task.db_tx,
      task.block,
      &previous_data.byron_block.as_ref().unwrap(),
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.byron_txs = result;
  };
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
    database_block: &BlockModel,
    readonly: bool,
) -> Result<Vec<TransactionModel>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            if main_block.body.tx_payload.is_empty() {
                return Ok(vec![]);
            }

            if readonly {
                let tx_hashes = main_block
                    .body
                    .tx_payload
                    .iter()
                    .map(|tx_body| {
                        blake2b256(&tx_body.transaction.encode_fragment().expect("")).to_vec()
                    })
                    .collect::<Vec<_>>();
                let txs = transactions_from_hashes(db_tx, tx_hashes.as_slice()).await;
                return txs;
            }

            let transaction_inserts =
                Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
                    |(idx, tx_body)| {
                        let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                        let tx_payload = tx_body.encode_fragment().unwrap();

                        TransactionActiveModel {
                            hash: Set(tx_hash.to_vec()),
                            block_id: Set(database_block.id),
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
