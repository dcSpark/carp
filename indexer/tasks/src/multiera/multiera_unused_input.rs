use super::{
    multiera_outputs::MultieraOutputTask, multiera_used_inputs::add_input_relations,
    relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraUnusedInputTask;
  configuration EmptyConfig;
  doc "Adds the unused inputs to the database (collateral inputs if tx succeeds, collateral inputs otherwise";
  era multiera;
  dependencies [MultieraOutputTask];
  read [multiera_txs];
  write [vkey_relation_map];
  should_add_task |block, _properties| {
    // if any txs has collateral defined, then it has some unused input (either collateral or main inputs if tx failed)
    block.1.txs().iter().any(|tx| tx.collateral().len() > 0)
  };
  execute |previous_data, task| handle_unused_input(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
  );
  merge_result |previous_data, _result| {
  };
}

type QueuedInputs = Vec<(Vec<OutputRef>, i64)>;

async fn handle_unused_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<(), DbErr> {
    let mut queued_unused_inputs = QueuedInputs::default();
    let txs = block.1.txs();

    for (tx_body, cardano_transaction) in txs.iter().zip(multiera_txs) {
        if !cardano_transaction.is_valid {
            let refs = tx_body.inputs().iter().map(|x| x.output_ref()).collect();
            queued_unused_inputs.push((refs, cardano_transaction.id))
        }

        if cardano_transaction.is_valid {
            // note: we consider collateral as just another kind of input instead of a separate table
            // you can use the is_valid field to know what kind of input it actually is
            let refs = tx_body
                .collateral()
                .iter()
                .map(|x| x.output_ref())
                .collect();
            queued_unused_inputs.push((refs, cardano_transaction.id))
        }
    }

    if !queued_unused_inputs.is_empty() {
        let outputs_for_inputs =
            crate::era_common::get_outputs_for_inputs(&queued_unused_inputs, db_tx).await?;
        let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

        add_input_relations(
            vkey_relation_map,
            &queued_unused_inputs,
            outputs_for_inputs
                .iter()
                .map(|(output, _)| output)
                .collect::<Vec<_>>()
                .as_slice(),
            &input_to_output_map,
        );
    }

    Ok(())
}
