use crate::dsl::task_macro::*;
use pallas::ledger::primitives::byron::{self, TxIn};

use super::byron_outputs::ByronOutputTask;
use crate::config::EmptyConfig::EmptyConfig;

carp_task! {
  name ByronInputTask;
  configuration EmptyConfig;
  doc "Adds the transaction inputs to the database";
  era byron;
  dependencies [ByronOutputTask];
  read [byron_txs];
  write [byron_inputs];
  should_add_task |block, _properties| {
    // recall: all txs must have at least 1 input
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_inputs(
      task.db_tx,
      task.block,
      previous_data.byron_txs.as_slice(),
  );
  merge_result |previous_data, result| {
    *previous_data.byron_inputs = result;
  };
}

async fn handle_inputs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    byron_txs: &[TransactionModel],
) -> Result<Vec<TransactionInputModel>, DbErr> {
    let flattened_inputs: Vec<(Vec<pallas::ledger::traverse::OutputRef>, i64)> = block
        .1
        .txs()
        .iter()
        .zip(byron_txs)
        .map(|(tx, cardano_tx_in_db)| {
            let inputs: Vec<pallas::ledger::traverse::OutputRef> =
                tx.inputs().iter().map(|x| x.output_ref()).collect();

            (inputs, cardano_tx_in_db.id)
        })
        .collect();

    let outputs_for_inputs =
        crate::era_common::get_outputs_for_inputs(&flattened_inputs, db_tx).await?;

    let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);
    let result =
        crate::era_common::insert_inputs(&flattened_inputs, &input_to_output_map, db_tx).await?;
    Ok(result)
}
