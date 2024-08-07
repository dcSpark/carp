use std::collections::BTreeMap;

use crate::dsl::database_task::BlockGlobalInfo;
use crate::era_common::input_from_pointer;
use crate::types::TxCredentialRelationValue;
use crate::{config::ReadonlyConfig::ReadonlyConfig, era_common::OutputWithTxData};
use cml_chain::{
    address::{BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    byron::ByronAddress,
};
use cml_core::serialization::{FromBytes, ToBytes};
use cml_crypto::RawBytesEncoding;
use cml_multi_era::utils::MultiEraTransactionInput;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};

use super::{multiera_used_outputs::MultieraOutputTask, relation_map::RelationMap};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraUsedInputTask;
  configuration ReadonlyConfig;
  doc "Adds the used inputs to the database (regular inputs in most cases, collateral inputs if tx fails)";
  era multiera;
  dependencies [MultieraOutputTask];
  read [multiera_txs];
  write [vkey_relation_map, multiera_used_inputs, multiera_used_inputs_to_outputs_map];
  should_add_task |block, _properties| {
    // txs always have at least one input (even if tx fails)
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_input(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &mut previous_data.vkey_relation_map,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_used_inputs = result.0;
    *previous_data.multiera_used_inputs_to_outputs_map = result.1;
  };
}

type QueuedInputs = Vec<(
    Vec<MultiEraTransactionInput>,
    i64, // tx_id
)>;

async fn handle_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
    readonly: bool,
) -> Result<
    (
        Vec<TransactionInputModel>,
        BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    ),
    DbErr,
> {
    let mut queued_inputs = QueuedInputs::default();
    let txs = block.1.transaction_bodies();

    for (tx_body, cardano_transaction) in txs.iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            let refs = tx_body.inputs();
            queued_inputs.push((refs, cardano_transaction.id));
        }

        if !cardano_transaction.is_valid {
            let refs = tx_body
                .collateral_inputs()
                .map(|collateral_inputs| {
                    collateral_inputs
                        .iter()
                        .cloned()
                        .map(MultiEraTransactionInput::Shelley)
                        .collect()
                })
                .unwrap_or_else(std::vec::Vec::new);
            queued_inputs.push((refs, cardano_transaction.id))
        }
    }

    match queued_inputs.is_empty() {
        true => Ok((vec![], BTreeMap::default())),
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
                    .map(|output| &output.model)
                    .collect::<Vec<_>>()
                    .as_slice(),
                &input_to_output_map,
                TxCredentialRelationValue::Input,
                TxCredentialRelationValue::InputStake,
            );
            if readonly {
                Ok((
                    input_from_pointer(
                        db_tx,
                        queued_inputs
                            .iter()
                            .flat_map(|pair| {
                                pair.0.iter().enumerate().zip(std::iter::repeat(pair.1))
                            })
                            .map(|((idx, _), tx_id)| (tx_id, idx))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await?,
                    input_to_output_map,
                ))
            } else {
                Ok((
                    crate::era_common::insert_inputs(&queued_inputs, &input_to_output_map, db_tx)
                        .await?,
                    input_to_output_map,
                ))
            }
        }
    }
}

pub fn add_input_relations(
    vkey_relation_map: &mut RelationMap,
    inputs: &[(Vec<MultiEraTransactionInput>, i64)],
    outputs: &[&TransactionOutputModel],
    input_to_output_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    input_relation: TxCredentialRelationValue,
    input_stake_relation: TxCredentialRelationValue,
) {
    let mut output_to_input_tx = BTreeMap::<i64, i64>::default();
    for input_tx_pair in inputs.iter() {
        for input in input_tx_pair.0.iter() {
            match input_to_output_map.get(&input.hash().unwrap().to_raw_bytes().to_vec()) {
                Some(entry_for_tx) => {
                    let output = &entry_for_tx[&(input.index().unwrap() as i64)];
                    output_to_input_tx.insert(output.model.id, input_tx_pair.1);
                }
                None => {
                    panic!(
                        "tx: {} index:{}",
                        input.hash().unwrap().to_hex(),
                        input.index().unwrap()
                    );
                }
            }
        }
    }

    outputs.iter().for_each(|&output| {
        match &cml_chain::transaction::TransactionOutput::from_bytes(output.payload.clone()) {
            Ok(payload) => {
                add_input_cred_relation(
                    vkey_relation_map,
                    output_to_input_tx[&output.id],
                    payload.address(),
                    input_relation,
                    input_stake_relation,
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
    addr: &cml_chain::address::Address,
    input_relation: TxCredentialRelationValue,
    input_stake_relation: TxCredentialRelationValue,
) {
    if let Some(base_addr) = BaseAddress::from_address(addr) {
        // Payment Key
        {
            vkey_relation_map.add_relation(tx_id, base_addr.payment.to_raw_bytes(), input_relation);
        }

        // Stake Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                base_addr.stake.to_raw_bytes(),
                input_stake_relation,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(addr) {
        vkey_relation_map.add_relation(tx_id, reward_addr.payment.to_raw_bytes(), input_relation);
    } else if ByronAddress::from_address(addr).is_some() {
        // Byron address has no credentials
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(addr) {
        vkey_relation_map.add_relation(
            tx_id,
            enterprise_addr.payment.to_raw_bytes(),
            input_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(addr) {
        vkey_relation_map.add_relation(tx_id, ptr_addr.payment.to_raw_bytes(), input_relation);
    } else {
        panic!(
            "Unexpected address type {}",
            hex::encode(addr.to_raw_bytes())
        );
    }
}
