use crate::{dsl::task_macro::*, era_common::transactions_from_hashes, utils::blake2b256};
use cml_core::serialization::ToBytes;
use cml_multi_era::byron::block::ByronBlock;
use cml_multi_era::MultiEraBlock;
use entity::sea_orm::Set;

use super::byron_block::ByronBlockTask;
use crate::config::PayloadAndReadonlyConfig::PayloadAndReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;

carp_task! {
  name ByronTransactionTask;
  configuration PayloadAndReadonlyConfig;
  doc "Adds the transactions in the block to the database";
  era byron;
  dependencies [ByronBlockTask];
  read [byron_block];
  write [byron_txs];
  should_add_task |block, _properties| {
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_tx(
      task.db_tx,
      task.block,
      &previous_data.byron_block.as_ref().unwrap(),
      task.config.readonly,
      task.config.include_payload
  );
  merge_result |previous_data, result| {
    *previous_data.byron_txs = result;
  };
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    database_block: &BlockModel,
    readonly: bool,
    include_payload: bool,
) -> Result<Vec<TransactionModel>, DbErr> {
    if block.1.is_empty() {
        return Ok(vec![]);
    }

    if readonly {
        let tx_hashes = match block.1 {
            MultiEraBlock::Byron(ByronBlock::Main(main)) => main
                .body
                .tx_payload
                .iter()
                .map(|tx| <[u8; 32]>::from(tx.byron_tx.hash()).to_vec())
                .collect::<Vec<Vec<u8>>>(),
            _ => vec![],
        };
        let txs = transactions_from_hashes(db_tx, &tx_hashes).await;
        return txs;
    }

    let transactions = match block.1 {
        MultiEraBlock::Byron(ByronBlock::Main(main)) => main.body.tx_payload.clone(),
        _ => vec![],
    };

    let transaction_inserts =
        Transaction::insert_many(transactions.iter().enumerate().map(|(idx, tx)| {
            let tx_payload = if include_payload {
                tx.to_bytes()
            } else {
                vec![]
            };

            TransactionActiveModel {
                hash: Set(<[u8; 32]>::from(tx.byron_tx.hash()).to_vec()),
                block_id: Set(database_block.id),
                tx_index: Set(idx as i32),
                payload: Set(tx_payload),
                is_valid: Set(true), // always true in Byron
                ..Default::default()
            }
        }))
        .exec_many_with_returning(db_tx)
        .await?;
    Ok(transaction_inserts)
}
