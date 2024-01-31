use crate::dsl::task_macro::*;
use cml_multi_era::byron::block::ByronBlock;
use cml_multi_era::byron::transaction::{ByronTx, ByronTxIn};
use cml_multi_era::utils::MultiEraTransactionInput;
use cml_multi_era::MultiEraBlock;

use super::byron_outputs::ByronOutputTask;
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;

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
    block: BlockInfo<'_, MultiEraBlock, BlockGlobalInfo>,
    byron_txs: &[TransactionModel],
) -> Result<Vec<TransactionInputModel>, DbErr> {
    let txs = match block.1 {
        MultiEraBlock::Byron(ByronBlock::Main(block)) => block.body.tx_payload.iter().map(|tx| {
            tx.byron_tx
                .inputs
                .iter()
                .cloned()
                .map(MultiEraTransactionInput::Byron)
        }),
        _ => {
            return Ok(vec![]);
        }
    };

    let flattened_inputs: Vec<(Vec<_>, i64)> = txs
        .zip(byron_txs)
        .map(|(inputs, cardano_tx_in_db)| (inputs.collect::<Vec<_>>(), cardano_tx_in_db.id))
        .collect();

    let outputs_for_inputs =
        crate::era_common::get_outputs_for_inputs(&flattened_inputs, db_tx).await?;

    let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);
    let result =
        crate::era_common::insert_inputs(&flattened_inputs, &input_to_output_map, db_tx).await?;
    Ok(result)
}
