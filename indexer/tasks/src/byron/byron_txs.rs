use entity::sea_orm::Set;
use pallas::ledger::primitives::{byron, Fragment};
use crate::{database_task::PrerunResult, task_macro::*, utils::blake2b256};

#[derive(Copy, Clone)]
pub struct ByronTransactionPrerunData();

carp_task! {
  name ByronTransactionTask;
  era byron;
  dependencies [];
  read [];
  write [byron_txs];
  should_add_task |block, properties| -> ByronTransactionPrerunData {
    PrerunResult::RunTaskWith(ByronTransactionPrerunData())
  };
  execute |previous_data, task| handle_tx(
      task.db_tx,
      task.block,
  );
  merge_result |previous_data, result| {
    *previous_data.byron_txs = result;
  };
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
) -> Result<Vec<TransactionModel>, DbErr> {
    match &block.1 {
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
                            block_id: Set(block.2.id),
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
