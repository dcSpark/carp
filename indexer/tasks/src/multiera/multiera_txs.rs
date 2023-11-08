use std::collections::BTreeSet;

use super::multiera_block::MultieraBlockTask;
use crate::config::PayloadAndReadonlyConfig::PayloadAndReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;
use crate::era_common::transactions_from_hashes;
use entity::sea_orm::{DatabaseTransaction, QueryOrder, Set};
use pallas::ledger::primitives::alonzo::{self};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::MultiEraBlock;

carp_task! {
  name MultieraTransactionTask;
  configuration PayloadAndReadonlyConfig;
  doc "Adds the transactions in the block to the database";
  era multiera;
  dependencies [MultieraBlockTask];
  read [multiera_block];
  write [multiera_txs];
  should_add_task |block, _properties| {
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_tx(
      task.db_tx,
      task.block,
      &previous_data.multiera_block.as_ref().unwrap(),
      task.config.readonly,
      task.config.include_payload
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_txs = result;
  };
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    database_block: &BlockModel,
    readonly: bool,
    include_payload: bool,
) -> Result<Vec<TransactionModel>, DbErr> {
    if readonly {
        let txs = transactions_from_hashes(
            db_tx,
            block
                .1
                .txs()
                .iter()
                .map(|tx_body| tx_body.hash().to_vec())
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await;
        return txs;
    }

    let txs: Vec<TransactionActiveModel> = block
        .1
        .txs()
        .iter()
        .enumerate()
        .map(|(idx, tx)| {
            let tx_payload = if include_payload { tx.encode() } else { vec![] };
            TransactionActiveModel {
                hash: Set(tx.hash().to_vec()),
                block_id: Set(database_block.id),
                tx_index: Set(idx as i32),
                payload: Set(tx_payload),
                is_valid: Set(tx.is_valid()),
                ..Default::default()
            }
        })
        .collect();

    if !txs.is_empty() {
        let insertions = Transaction::insert_many(txs)
            .exec_many_with_returning(db_tx)
            .await?;
        Ok(insertions)
    } else {
        Ok(vec![])
    }
}
