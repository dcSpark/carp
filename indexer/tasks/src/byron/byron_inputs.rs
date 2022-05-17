use crate::{dsl::default_impl::has_transaction_byron, dsl::task_macro::*};
use pallas::ledger::primitives::byron::{self, TxIn};

use super::byron_outputs::ByronOutputTask;

carp_task! {
  name ByronInputTask;
  doc "Adds the transaction inputs to the database";
  era byron;
  dependencies [ByronOutputTask];
  read [byron_txs];
  write [byron_inputs];
  should_add_task |block, _properties| {
    // recall: all txs must have at least 1 input
    has_transaction_byron(block.1)
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
    block: BlockInfo<'_, byron::Block>,
    byron_txs: &[TransactionModel],
) -> Result<Vec<TransactionInputModel>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            let all_inputs: Vec<(
                Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
                i64,
            )> = main_block
                .body
                .tx_payload
                .iter()
                .zip(byron_txs)
                .map(|(tx_payload, cardano_tx_in_db)| {
                    let inputs: Vec<pallas::ledger::primitives::alonzo::TransactionInput> =
                        tx_payload
                            .transaction
                            .inputs
                            .iter()
                            .map(byron_input_to_alonzo)
                            .collect();
                    (inputs, cardano_tx_in_db.id)
                })
                .collect();

            let flattened_inputs: Vec<(
                &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
                i64,
            )> = all_inputs
                .iter()
                .map(|inputs| (&inputs.0, inputs.1))
                .collect();
            let outputs_for_inputs =
                crate::era_common::get_outputs_for_inputs(&flattened_inputs, db_tx).await?;

            let input_to_output_map =
                crate::era_common::gen_input_to_output_map(&outputs_for_inputs);
            let result =
                crate::era_common::insert_inputs(&flattened_inputs, &input_to_output_map, db_tx)
                    .await?;
            Ok(result)
        }
    }
}

fn byron_input_to_alonzo(input: &TxIn) -> pallas::ledger::primitives::alonzo::TransactionInput {
    match input {
        TxIn::Variant0(wrapped) => pallas::ledger::primitives::alonzo::TransactionInput {
            transaction_id: wrapped.0 .0,
            index: wrapped.0 .1 as u64,
        },
        TxIn::Other(index, tx_hash) => {
            // Note: Pallas uses "other" to future proof itself against changes in the binary spec
            todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
        }
    }
}
