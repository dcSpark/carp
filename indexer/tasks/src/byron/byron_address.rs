use pallas::ledger::primitives::{
    byron::{self},
    Fragment,
};

use super::byron_txs::ByronTransactionTask;
use crate::{database_task::PrerunResult, task_macro::*};

#[derive(Copy, Clone)]
pub struct ByronAddressPrerunData();

carp_task! {
  name ByronAddressTask;
  era byron;
  dependencies [ByronTransactionTask];
  read [byron_txs];
  write [byron_addresses];
  should_add_task |_block, _properties| -> ByronAddressPrerunData {
    PrerunResult::RunTaskWith(ByronAddressPrerunData())
  };
  execute |previous_data, task| handle_outputs(
      task.db_tx,
      task.block,
      previous_data.byron_txs.as_slice(),
  );
  merge_result |previous_data, result| {
    *previous_data.byron_addresses = result;
  };
}

async fn handle_outputs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
    byron_txs: &[TransactionModel],
) -> Result<BTreeMap<Vec<u8>, AddressInBlock>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(BTreeMap::<Vec<u8>, AddressInBlock>::default()),
        byron::Block::MainBlock(main_block) => {
            let tx_outputs: Vec<_> = main_block
                .body
                .tx_payload
                .iter()
                .map(|payload| &payload.transaction.outputs)
                .zip(byron_txs)
                .collect();

            if tx_outputs.is_empty() {
                return Ok(BTreeMap::<Vec<u8>, AddressInBlock>::default());
            }
            // insert addresses
            let address_map = crate::era_common::insert_addresses(
                &tx_outputs
                    .iter()
                    .flat_map(|pair| pair.0.iter())
                    .map(|output| output.address.encode_fragment().unwrap())
                    .collect(),
                db_tx,
            )
            .await?;

            Ok(address_map)
        }
    }
}
