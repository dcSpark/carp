use crate::config::PayloadConfig::PayloadConfig;
use crate::dsl::task_macro::*;
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;

carp_task! {
  name GenesisBlockTask;
  configuration PayloadConfig;
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
      task.config.include_payload
  );
  merge_result |previous_data, result| {
    *previous_data.genesis_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, GenesisData, BlockGlobalInfo>,
    include_payload: bool,
) -> Result<BlockModel, DbErr> {
    let genesis_hash = block.1.genesis_prev.to_bytes();

    let block_payload = if include_payload {
        hex::decode(block.0).unwrap()
    } else {
        vec![]
    };
    let block = BlockActiveModel {
        era: Set(EraValue::Byron.into()),
        hash: Set(genesis_hash),
        // note: strictly speaking, the epoch, height, etc. isn't defined for the genesis block
        // since it comes before the first Epoch Boundary Block (EBB)
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        payload: Set(block_payload),
        ..Default::default()
    };

    block.insert(db_tx).await
}
