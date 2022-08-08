use std::collections::BTreeMap;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::types::TxCredentialRelationValue;
use cardano_multiplatform_lib::{
    address::{BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    byron::ByronAddress,
};
use entity::sea_orm::QueryOrder;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use pallas::ledger::traverse::{MultiEraBlock, MultiEraInput, OutputRef};

use super::{
    multiera_used_inputs::add_input_relations, multiera_used_outputs::MultieraOutputTask,
    relation_map::RelationMap,
};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraReferenceInputTask;
  configuration ReadonlyConfig;
  doc "Adds the reference inputs to the database. Data is still written if the tx fails";
  era multiera;
  dependencies [MultieraOutputTask];
  read [multiera_txs];
  write [vkey_relation_map];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.reference_inputs().is_some())
  };
  execute |previous_data, task| handle_input(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
      task.config.readonly
  );
  merge_result |previous_data, result| {
  };
}

type QueuedInputs = Vec<(
    Vec<OutputRef>,
    i64, // tx_id
)>;

async fn handle_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
    readonly: bool,
) -> Result<Vec<TransactionReferenceInputModel>, DbErr> {
    let mut queued_inputs = QueuedInputs::default();
    let txs = block.1.txs();

    for (tx_body, cardano_transaction) in txs.iter().zip(multiera_txs) {
        let refs = tx_body
            .reference_inputs()
            .iter()
            .map(|x| x.output_ref())
            .collect();
        queued_inputs.push((refs, cardano_transaction.id));
    }

    match queued_inputs.is_empty() {
        true => Ok(vec![]),
        false => {
            let outputs_for_inputs =
                crate::era_common::get_outputs_for_inputs(&queued_inputs, db_tx).await?;
            let input_to_output_map =
                crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

            add_input_relations(
                vkey_relation_map,
                &queued_inputs,
                outputs_for_inputs
                    .iter()
                    .map(|(output, _)| output)
                    .collect::<Vec<_>>()
                    .as_slice(),
                &input_to_output_map,
                TxCredentialRelationValue::ReferenceInput,
                TxCredentialRelationValue::ReferenceInputStake,
            );
            if readonly {
                Ok(reference_input_from_pointer(
                    db_tx,
                    queued_inputs
                        .iter()
                        .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
                        .map(|((idx, _), tx_id)| (tx_id, idx))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .await?)
            } else {
                Ok(insert_reference_inputs(&queued_inputs, &input_to_output_map, db_tx).await?)
            }
        }
    }
}

pub async fn insert_reference_inputs(
    inputs: &[(Vec<pallas::ledger::traverse::OutputRef>, i64)],
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, &TransactionOutputModel>>,
    txn: &DatabaseTransaction,
) -> Result<Vec<TransactionReferenceInputModel>, DbErr> {
    // avoid querying the DB if there were no inputs
    let has_input = inputs.iter().any(|input| !input.0.is_empty());
    if !has_input {
        return Ok(vec![]);
    }

    let result = TransactionReferenceInput::insert_many(
        inputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(|((idx, input), tx_id)| {
                let output = input_to_output_map[&input.hash().to_vec()][&(input.index() as i64)];
                TransactionReferenceInputActiveModel {
                    utxo_id: Set(output.id),
                    address_id: Set(output.address_id),
                    tx_id: Set(tx_id),
                    input_index: Set(idx as i32),
                    ..Default::default()
                }
            }),
    )
    .exec_many_with_returning(txn)
    .await?;

    Ok(result)
}

pub async fn reference_input_from_pointer(
    db_tx: &DatabaseTransaction,
    pointers: &[(i64 /* txid */, usize /* input index */)],
) -> Result<Vec<TransactionReferenceInputModel>, DbErr> {
    // https://github.com/dcSpark/carp/issues/46
    let mut input_conditions = Condition::any();
    for (tx_id, input_index) in pointers.iter() {
        input_conditions = input_conditions.add(
            Condition::all()
                .add(TransactionReferenceInputColumn::TxId.eq(*tx_id))
                .add(TransactionReferenceInputColumn::InputIndex.eq(*input_index as i32)),
        );
    }

    let inputs = TransactionReferenceInput::find()
        .filter(input_conditions)
        .order_by_asc(TransactionReferenceInputColumn::Id)
        .all(db_tx)
        .await?;
    Ok(inputs)
}
