use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::task_macro::*;
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;
use pallas::ledger::primitives::{byron, Fragment};

use super::byron_block::ByronBlockTask;

carp_task! {
  name ByronBlockMinterTask;
  configuration ReadonlyConfig;
  doc "Adds the minter of a block to the database";
  era byron;
  dependencies [ByronBlockTask];
  read [byron_block];
  write [];
  should_add_task |_block, _properties| {
    true
  };
  execute |previous_data, task| handle_block(
      task.db_tx,
      task.block,
      &previous_data.byron_block.as_ref().unwrap(),
      task.config.readonly
  );
  merge_result |previous_data, _result| {
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    database_block: &BlockModel,
    readonly: bool,
) -> Result<BlockMinterModel, DbErr> {
    if readonly {
        let entry = BlockMinter::find()
            .filter(BlockMinterColumn::Id.eq(database_block.id))
            .one(db_tx)
            .await?;
        return Ok(match entry {
            None => {
                panic!(
                    "Block not found in database: {}",
                    hex::encode(block.1.hash())
                );
            }
            Some(block_minter) => block_minter,
        });
    }

    let block_minter = BlockMinterActiveModel {
        id: Set(database_block.id),
        key: Set(block.1.as_byron().unwrap().header.consensus_data.1.to_vec()),
    };

    Ok(block_minter.insert(db_tx).await?)
}
