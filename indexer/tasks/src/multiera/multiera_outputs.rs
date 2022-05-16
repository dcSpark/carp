use std::{
    collections::BTreeMap,
};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::alonzo::{self, TransactionBody, TransactionBodyComponent};
use pallas::ledger::primitives::Fragment;

use crate::{
    era_common::{get_truncated_address},
};

use super::multiera_address::MultieraAddressTask;

use crate::{database_task::PrerunResult, task_macro::*};

#[derive(Copy, Clone)]
pub struct MultieraOutputPrerunData();

carp_task! {
  name MultieraOutputTask;
  era multiera;
  dependencies [MultieraAddressTask];
  read [multiera_txs, multiera_addresses];
  write [multiera_outputs];
  should_add_task |_block, _properties| -> MultieraOutputPrerunData {
    PrerunResult::RunTaskWith(MultieraOutputPrerunData())
  };
  execute |previous_data, task| handle_output(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_addresses,
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_outputs = result;
  };
}

struct QueuedOutput {
    // note: no need to use a map type
    // because the pair <tx_id, idx> should only ever be inserted once
    tx_id: i64,
    idx: usize,
    payload: Vec<u8>,
    address: Vec<u8>, // pallas::crypto::hash::Hash<32>
}

async fn handle_output(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let mut queued_output = Vec::<QueuedOutput>::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            #[allow(clippy::single_match)]
            match component {
                TransactionBodyComponent::Outputs(outputs) => {
                    for (idx, output) in outputs.iter().enumerate() {
                        queue_output(
                            &mut queued_output,
                            tx_body,
                            cardano_transaction.id,
                            output,
                            idx,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    Ok(insert_outputs(addresses, &queued_output, db_tx).await?)
}

fn queue_output(
    queued_output: &mut Vec<QueuedOutput>,
    tx_body: &TransactionBody,
    tx_id: i64,
    output: &alonzo::TransactionOutput,
    idx: usize,
) {
    use cardano_multiplatform_lib::address::Address;
    let addr = Address::from_bytes(output.address.to_vec())
        .map_err(|e| panic!("{:?}{:?}", e, tx_body.to_hash().to_vec()))
        .unwrap();

    queued_output.push(QueuedOutput {
        payload: output.encode_fragment().unwrap(),
        address: addr.to_bytes(),
        tx_id,
        idx,
    });
}

async fn insert_outputs(
    address_to_model_map: &BTreeMap<Vec<u8>, AddressInBlock>,
    queued_output: &[QueuedOutput],
    txn: &DatabaseTransaction,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    if queued_output.is_empty() {
        return Ok(vec![]);
    };

    Ok(
        TransactionOutput::insert_many(queued_output.iter().map(|entry| {
            TransactionOutputActiveModel {
                address_id: Set(address_to_model_map
                    .get(get_truncated_address(&entry.address))
                    .unwrap()
                    .model
                    .id),
                tx_id: Set(entry.tx_id),
                payload: Set(entry.payload.clone()),
                output_index: Set(entry.idx as i32),
                ..Default::default()
            }
        }))
        .exec_many_with_returning(txn)
        .await?,
    )
}
