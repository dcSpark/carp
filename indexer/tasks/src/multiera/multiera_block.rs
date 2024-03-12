use crate::config::PayloadAndReadonlyConfig::PayloadAndReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;
use crate::era_common::block_from_hash;
use crate::utils::blake2b256;
use entity::sea_orm::{DatabaseTransaction, Set};

carp_task! {
  name MultieraBlockTask;
  configuration PayloadAndReadonlyConfig;
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
      task.config.readonly,
      task.config.include_payload
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_block = Some(result);
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    readonly: bool,
    include_payload: bool,
) -> Result<BlockModel, DbErr> {
    let hash = block.1.hash();
    if readonly {
        return block_from_hash(db_tx, &hash).await;
    }
    let block_payload = if include_payload {
        hex::decode(block.0).unwrap()
    } else {
        vec![]
    };
    let block = BlockActiveModel {
        era: Set(block.2.era.into()),
        hash: Set(hash.to_vec()),
        height: Set(block.1.header().block_number() as i32),
        epoch: Set(block.2.epoch.unwrap() as i32),
        slot: Set(block.1.header().slot() as i32),
        payload: Set(Some(block_payload)),
        tx_count: Set(block_tx_count(block.1) as i32),
        ..Default::default()
    };
    block.insert(db_tx).await
}

fn block_tx_count(block: &cml_multi_era::MultiEraBlock) -> usize {
    match block {
        cml_multi_era::MultiEraBlock::Byron(
            cml_multi_era::byron::block::ByronBlock::EpochBoundary(_),
        ) => 0,
        cml_multi_era::MultiEraBlock::Byron(cml_multi_era::byron::block::ByronBlock::Main(
            block,
        )) => block.body.tx_payload.len(),
        cml_multi_era::MultiEraBlock::Shelley(block) => block.transaction_bodies.len(),
        cml_multi_era::MultiEraBlock::Allegra(block) => block.transaction_bodies.len(),
        cml_multi_era::MultiEraBlock::Mary(block) => block.transaction_bodies.len(),
        cml_multi_era::MultiEraBlock::Alonzo(block) => block.transaction_bodies.len(),
        cml_multi_era::MultiEraBlock::Babbage(block) => block.transaction_bodies.len(),
        cml_multi_era::MultiEraBlock::Conway(block) => block.transaction_bodies.len(),
    }
}
