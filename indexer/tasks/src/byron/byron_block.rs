use crate::config::PayloadAndReadonlyConfig::PayloadAndReadonlyConfig;
use crate::dsl::task_macro::*;
use crate::era_common::block_from_hash;
use cml_core::serialization::ToBytes;
use cml_crypto::{blake2b256, RawBytesEncoding};
use cml_multi_era::byron::block::ByronBlock;
use cml_multi_era::MultiEraBlock;
use entity::{block::EraValue, sea_orm::Set};
use hex::ToHex;

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
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    readonly: bool,
    include_payload: bool,
) -> Result<BlockModel, DbErr> {
    let hash = block.1.hash();

    if readonly {
        return block_from_hash(db_tx, &hash).await;
    }

    let (tx_count, block_epoch) = match block.1 {
        MultiEraBlock::Byron(byron) => match byron {
            ByronBlock::EpochBoundary(byron) => (0, byron.header.consensus_data.epoch_id),
            ByronBlock::Main(byron) => (
                byron.body.tx_payload.len(),
                byron.header.consensus_data.byron_slot_id.epoch,
            ),
        },
        _ => {
            return Err(DbErr::Custom("Non-byron block in byron task".to_string()));
        }
    };

    let block_payload = if include_payload {
        hex::decode(block.0).unwrap()
    } else {
        vec![]
    };
    let block = BlockActiveModel {
        era: Set(EraValue::Byron.into()),
        hash: Set(hash.to_vec()),
        height: Set(block.1.header().block_number() as i32),
        epoch: Set(block_epoch as i32),
        slot: Set(block.1.header().slot() as i32),
        payload: Set(Some(block_payload)),
        tx_count: Set(tx_count as i32),
        ..Default::default()
    };

    block.insert(db_tx).await
}
