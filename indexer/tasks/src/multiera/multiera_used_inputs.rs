use std::collections::BTreeMap;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::era_common::input_from_pointer;
use crate::{dsl::default_impl::has_transaction_multiera, types::TxCredentialRelationValue};
use cardano_multiplatform_lib::address::{
    BaseAddress, ByronAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};

use super::{multiera_outputs::MultieraOutputTask, relation_map::RelationMap};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraUsedInputTask;
  configuration ReadonlyConfig;
  doc "Adds the used inputs to the database (regular inputs in most cases, collateral inputs if tx fails";
  era multiera;
  dependencies [MultieraOutputTask];
  read [multiera_txs];
  write [vkey_relation_map, multiera_used_inputs];
  should_add_task |block, _properties| {
    // txs always have at least one input (even if tx fails)
    has_transaction_multiera(block.1)
  };
  execute |previous_data, task| handle_input(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_used_inputs = result;
  };
}

type QueuedInputs<'a> = Vec<(
    &'a Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    i64, // tx_id
)>;

async fn handle_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
    readonly: bool,
) -> Result<Vec<TransactionInputModel>, DbErr> {
    let mut queued_inputs = QueuedInputs::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) if cardano_transaction.is_valid => {
                    queued_inputs.push((inputs, cardano_transaction.id))
                }
                TransactionBodyComponent::Collateral(inputs) if !cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    queued_inputs.push((inputs, cardano_transaction.id))
                }
                _ => (),
            };
        }
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
            );
            if readonly {
                Ok(input_from_pointer(
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
                Ok(
                    crate::era_common::insert_inputs(&queued_inputs, &input_to_output_map, db_tx)
                        .await?,
                )
            }
        }
    }
}

pub fn add_input_relations(
    vkey_relation_map: &mut RelationMap,
    inputs: &[(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )],
    outputs: &[&TransactionOutputModel],
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, &TransactionOutputModel>>,
) {
    let mut output_to_input_tx = BTreeMap::<i64, i64>::default();
    for input_tx_pair in inputs.iter() {
        for input in input_tx_pair.0.iter() {
            match input_to_output_map.get(&input.transaction_id.to_vec()) {
                Some(entry_for_tx) => {
                    let output_id = entry_for_tx[&(input.index as i64)];
                    output_to_input_tx.insert(output_id.id, input_tx_pair.1);
                }
                None => {
                    println!("tx: {}", hex::encode(input.transaction_id));
                    panic!();
                }
            }
        }
    }

    outputs.iter().for_each(|&output| {
        match &cardano_multiplatform_lib::TransactionOutput::from_bytes(output.payload.clone()) {
            Ok(payload) => {
                add_input_cred_relation(
                    vkey_relation_map,
                    output_to_input_tx[&output.id],
                    &payload.address(),
                    TxCredentialRelationValue::Input,
                    TxCredentialRelationValue::InputStake,
                );
            }
            Err(_e) => {
                // https://github.com/dcSpark/cardano-multiplatform-lib/issues/61
            }
        };
    });
}

fn add_input_cred_relation(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    addr: &cardano_multiplatform_lib::address::Address,
    input_relation: TxCredentialRelationValue,
    input_stake_relation: TxCredentialRelationValue,
) {
    if let Some(base_addr) = BaseAddress::from_address(addr) {
        // Payment Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.payment_cred().to_bytes(),
                input_relation,
            );
        }

        // Stake Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.stake_cred().to_bytes(),
                input_stake_relation,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &reward_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if ByronAddress::from_address(addr).is_some() {
        // Byron address has no credentials
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &enterprise_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(addr) {
        vkey_relation_map.add_relation(tx_id, &ptr_addr.payment_cred().to_bytes(), input_relation);
    } else {
        panic!("Unexpected address type {}", hex::encode(addr.to_bytes()));
    }
}
