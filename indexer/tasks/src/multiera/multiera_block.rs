use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;
use crate::era_common::block_from_hash;
use crate::utils::blake2b256;
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::primitives::alonzo::{self};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::MultiEraBlock;

carp_task! {
  name MultieraBlockTask;
  configuration ReadonlyConfig;
  doc "Adds the block to the database";
  era multiera;
  dependencies [];
  read [];
  write [multiera_block];
  should_add_task |_block, _properties| {
    true
  };
  execute |previous_data, task| handle_block(
      task.db_tx,
      task.block,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    readonly: bool,
) -> Result<BlockModel, DbErr> {
    let hash = blake2b256(block.1.header().cbor());
    if readonly {
        return block_from_hash(db_tx, &hash).await;
    }
    let block = BlockActiveModel {
        era: Set(block.2.era.into()),
        hash: Set(hash.to_vec()),
        height: Set(block.1.number() as i32),
        epoch: Set(block.2.epoch.unwrap() as i32),
        slot: Set(block.1.slot() as i32),
        ..Default::default()
    };
    block.insert(db_tx).await
}
