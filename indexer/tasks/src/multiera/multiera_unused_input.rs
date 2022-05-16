use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};

use super::{
    multiera_outputs::MultieraOutputTask, multiera_used_inputs::add_input_relations,
    relation_map::RelationMap,
};

use crate::{database_task::PrerunResult, task_macro::*};

#[derive(Copy, Clone)]
pub struct MultieraUnusedInputPrerunData();

carp_task! {
  name MultieraUnusedInputTask;
  era multiera;
  dependencies [MultieraOutputTask];
  read [multiera_txs];
  write [vkey_relation_map];
  should_add_task |block, properties| -> MultieraUnusedInputPrerunData {
    PrerunResult::RunTaskWith(MultieraUnusedInputPrerunData())
  };
  execute |previous_data, task| handle_unused_input(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
  );
  merge_result |previous_data, result| {
  };
}


type QueuedInputs<'a> = Vec<(
    &'a Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    i64,
)>;

async fn handle_unused_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<(), DbErr> {
    let mut queued_unused_inputs = QueuedInputs::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) if !cardano_transaction.is_valid => {
                    queued_unused_inputs.push((inputs, cardano_transaction.id))
                }
                TransactionBodyComponent::Collateral(inputs) if cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    queued_unused_inputs.push((inputs, cardano_transaction.id))
                }
                _ => (),
            };
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
