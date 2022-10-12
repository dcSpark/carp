use pallas::ledger::primitives::{
    byron::{self},
    Fragment,
};

use super::byron_txs::ByronTransactionTask;
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::dsl::task_macro::*;

carp_task! {
  name ByronAddressTask;
  configuration EmptyConfig;
  doc "Adds the address raw bytes to the database";
  era byron;
  dependencies [ByronTransactionTask];
  read [byron_txs];
  write [byron_addresses];
  should_add_task |block, _properties| {
    // recall: txs may have no outputs if they just burn all inputs as fee
    match block.1 {
        MultiEraBlock::Byron(main_block) => {
            main_block
                .body
                .tx_payload.iter().any(|payload| payload.transaction.outputs.len() > 0)
        }
        _ => false,
    }
  };
  execute |previous_data, task| handle_addresses(
      task.db_tx,
      task.block,
      previous_data.byron_txs.as_slice(),
  );
  merge_result |previous_data, result| {
    *previous_data.byron_addresses = result;
  };
}

async fn handle_addresses(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    byron_txs: &[TransactionModel],
) -> Result<BTreeMap<Vec<u8>, AddressInBlock>, DbErr> {
    match &block.1 {
        MultiEraBlock::Byron(main_block) => {
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

            let mut queued_address = BTreeMap::<Vec<u8>, i64>::default();
            for (address, tx_id) in tx_outputs
                .iter()
                .flat_map(|pair| pair.0.iter().zip(std::iter::repeat(pair.1.id)))
                .map(|(output, tx_id)| (output.address.encode_fragment().unwrap(), tx_id))
            {
                // we want to keep track of the first tx for each address
                queued_address.entry(address).or_insert(tx_id);
            }
            // insert addresses
            let address_map = crate::era_common::insert_addresses(&queued_address, db_tx).await?;

            Ok(address_map)
        }
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        _ => Ok(BTreeMap::<Vec<u8>, AddressInBlock>::default()),
    }
}
