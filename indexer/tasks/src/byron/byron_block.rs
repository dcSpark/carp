use crate::config::PayloadAndReadonlyConfig::PayloadAndReadonlyConfig;
use crate::dsl::task_macro::*;
use crate::era_common::block_from_hash;
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;
use pallas::ledger::primitives::{byron, Fragment};

carp_task! {
  name ByronBlockTask;
  configuration PayloadAndReadonlyConfig;
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
      task.config.readonly,
      task.config.include_payload
  );
  merge_result |previous_data, result| {
    *previous_data.byron_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    readonly: bool,
    include_payload: bool,
) -> Result<BlockModel, DbErr> {
    let hash = block.1.hash().to_vec();
    if readonly {
        return block_from_hash(db_tx, &hash).await;
    }

    let block_payload = if include_payload {
        hex::decode(block.0).unwrap()
    } else {
        vec![]
    };
    let block = BlockActiveModel {
        era: Set(EraValue::Byron.into()),
        hash: Set(hash),
        height: Set(block.1.number() as i32),
        epoch: Set(block.1.header().as_byron().unwrap().consensus_data.0.epoch as i32),
        slot: Set(block.1.slot() as i32),
        payload: Set(Some(block_payload)),
        ..Default::default()
    };

    block.insert(db_tx).await
}
