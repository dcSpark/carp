use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::task_macro::*;
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;

carp_task! {
  name GenesisBlockTask;
  configuration EmptyConfig;
  doc "Adds the block to the database";
  era genesis;
  dependencies [];
  read [];
  write [genesis_block];
  should_add_task |_block, _properties| {
    true
  };
  execute |previous_data, task| handle_block(
      task.db_tx,
      task.block,
  );
  merge_result |previous_data, result| {
    *previous_data.genesis_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block_info: BlockInfo<'_, GenesisData>,
) -> Result<BlockModel, DbErr> {
    let genesis_hash = block_info.1.genesis_prev.to_bytes();

    let block_active_model = BlockActiveModel {
        era: Set(EraValue::Byron.into()),
        hash: Set(genesis_hash),
        // note: strictly speaking, the epoch, height, etc. isn't defined for the genesis block
        // since it comes before the first Epoch Boundary Block (EBB)
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        ..Default::default()
    };

    let block_model = block_active_model.insert(db_tx).await?;

    Ok(block_model)
}
