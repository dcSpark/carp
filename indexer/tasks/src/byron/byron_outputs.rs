use super::byron_address::ByronAddressTask;
use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use crate::{dsl::task_macro::*, era_common::get_truncated_address};
use cml_chain::byron::ByronTxOut;
use cml_core::serialization::ToBytes;
use cml_multi_era::byron::block::ByronBlock;
use cml_multi_era::MultiEraBlock;
use entity::sea_orm::Set;

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
    block.1.transaction_bodies().iter().any(|tx| !tx.outputs().is_empty())
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
    let tx_outputs: Vec<_> = match block.1 {
        MultiEraBlock::Byron(ByronBlock::Main(block)) => block
            .body
            .tx_payload
            .iter()
            .map(|payload| &payload.byron_tx.outputs)
            .zip(byron_txs)
            .collect(),
        _ => return Ok(vec![]),
    };

    if tx_outputs.is_empty() {
        return Ok(vec![]);
    }

    // note: outputs have to be added before inputs
    insert_byron_outputs(db_tx, byron_addresses, &tx_outputs).await
}

async fn insert_byron_outputs(
    txn: &DatabaseTransaction,
    address_map: &BTreeMap<Vec<u8>, AddressInBlock>,
    outputs: &[(&Vec<ByronTxOut>, &TransactionModel)],
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let result = TransactionOutput::insert_many(
        outputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(
                |((output_index, output), tx_id)| TransactionOutputActiveModel {
                    payload: Set(output.to_bytes()),
                    address_id: Set(address_map
                        .get(get_truncated_address(&output.address.to_bytes()))
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
