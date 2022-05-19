use crate::dsl::task_macro::*;
use crate::{dsl::default_impl::ReadonlyConfig, era_common::block_from_hash};
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;
use pallas::ledger::primitives::{byron, Fragment};

carp_task! {
  name ByronBlockTask;
  configuration ReadonlyConfig;
  doc "Adds the block to the database";
  era byron;
  dependencies [];
  read [];
  write [byron_block];
  should_add_task |_block, _properties| {
    true
  };
  execute |previous_data, task| handle_block(
      task.db_tx,
      task.block,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.byron_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
    readonly: bool,
) -> Result<BlockModel, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => panic!("We don't support EBBs"),
        byron::Block::MainBlock(main_block) => {
            let hash = main_block.header.to_hash().to_vec();
            if readonly {
                return block_from_hash(db_tx, &hash).await;
            }

            let block = BlockActiveModel {
                era: Set(EraValue::Byron.into()),
                hash: Set(hash),
                height: Set(main_block.header.consensus_data.2[0] as i32),
                epoch: Set(main_block.header.consensus_data.0.epoch as i32),
                slot: Set(main_block.header.consensus_data.0.to_abs_slot() as i32),
                ..Default::default()
            };
            Ok(block.insert(db_tx).await?)
        }
    }
}
