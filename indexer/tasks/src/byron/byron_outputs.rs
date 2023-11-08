use super::byron_address::ByronAddressTask;
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::{dsl::task_macro::*, era_common::get_truncated_address};
use entity::sea_orm::Set;
use pallas::ledger::traverse::MultiEraOutput;
use pallas::{
    codec::utils::MaybeIndefArray,
    ledger::primitives::{
        byron::{self, TxOut},
        Fragment,
    },
};

carp_task! {
  name ByronOutputTask;
  configuration EmptyConfig;
  doc "Adds the transaction outputs to the database";
  era byron;
  dependencies [ByronAddressTask];
  read [byron_txs, byron_addresses];
  write [byron_outputs];
  should_add_task |block, _properties| {
    // recall: txs may have no outputs if they just burn all inputs as fee
    block.1.txs().iter().any(|tx| tx.outputs().len() > 0)
  };
  execute |previous_data, task| handle_outputs(
      task.db_tx,
      task.block,
      previous_data.byron_txs.as_slice(),
      &previous_data.byron_addresses,
  );
  merge_result |previous_data, result| {
    *previous_data.byron_outputs = result;
  };
}

async fn handle_outputs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    byron_txs: &[TransactionModel],
    byron_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let tx_outputs: Vec<_> = block
        .1
        .as_byron()
        .unwrap()
        .body
        .tx_payload
        .iter()
        .map(|payload| &payload.transaction.outputs)
        .zip(byron_txs)
        .collect();

    if tx_outputs.is_empty() {
        return Ok(vec![]);
    }

    // note: outputs have to be added before inputs
    insert_byron_outputs(db_tx, byron_addresses, &tx_outputs).await
}

async fn insert_byron_outputs(
    txn: &DatabaseTransaction,
    address_map: &BTreeMap<Vec<u8>, AddressInBlock>,
    outputs: &[(&MaybeIndefArray<TxOut>, &TransactionModel)],
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let result = TransactionOutput::insert_many(
        outputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(
                |((output_index, output), tx_id)| TransactionOutputActiveModel {
                    payload: Set(output.encode_fragment().unwrap()),
                    address_id: Set(address_map
                        .get(get_truncated_address(
                            &output.address.encode_fragment().unwrap(),
                        ))
                        .unwrap()
                        .model
                        .id),
                    tx_id: Set(tx_id.id),
                    output_index: Set(output_index as i32),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(txn)
    .await?;

    Ok(result)
}
