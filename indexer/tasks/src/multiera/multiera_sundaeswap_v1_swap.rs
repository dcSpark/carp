use super::dex::common::{handle_swap, PoolType};
use super::multiera_used_inputs::MultieraUsedInputTask;
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::task_macro::*;

carp_task! {
  name MultieraSundaeSwapV1SwapTask;
  configuration EmptyConfig;
  doc "Adds SundaeSwap V1 swaps to the database";
  era multiera;
  dependencies [MultieraUsedInputTask];
  read [multiera_txs, multiera_addresses, multiera_used_inputs_to_outputs_map];
  write [];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.outputs().len() > 0)
  };
  execute |previous_data, task| handle_swap(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_addresses,
      &previous_data.multiera_used_inputs_to_outputs_map,
      PoolType::SundaeSwapV1,
  );
  merge_result |previous_data, _result| {
  };
}
