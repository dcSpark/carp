use cml_chain::auxdata::metadata::Metadata;
use cml_chain::auxdata::AuxiliaryData;
use cml_core::serialization::Serialize;
use std::collections::BTreeMap;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use entity::sea_orm::QueryOrder;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};

use super::multiera_txs::MultieraTransactionTask;

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraMetadataTask;
  configuration ReadonlyConfig;
  doc "Adds the transaction metadata to the database as a series of <metadata_label, cbor> pair";
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_txs];
  write [multiera_metadata];
  should_add_task |block, _properties| {
    !block.1.auxiliary_data_set().is_empty()
  };
  execute |previous_data, task| handle_metadata(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_metadata = result;
  };
}

async fn handle_metadata(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    readonly: bool,
) -> Result<Vec<TransactionMetadataModel>, DbErr> {
    if readonly {
        return TransactionMetadata::find()
            .filter(TransactionMetadataColumn::TxId.is_in(multiera_txs.iter().map(|tx| tx.id)))
            .order_by_asc(TransactionMetadataColumn::Id)
            .all(db_tx)
            .await;
    }

    let mut metadata_map = BTreeMap::<i64 /* id */, Metadata>::default();

    let tx_aux_data = block.1.auxiliary_data_set();

    for (idx, metadata) in tx_aux_data.iter() {
        let tx_id = &multiera_txs[*idx as usize].id;
        let meta = match metadata {
            AuxiliaryData::Conway(data) => match &data.metadata {
                None => continue,
                Some(metadata) => metadata.clone(),
            },
            AuxiliaryData::Shelley(data) => data.clone(),
            AuxiliaryData::ShelleyMA(data) => data.transaction_metadata.clone(),
        };

        if !meta.is_empty() {
            metadata_map.insert(*tx_id, meta);
        }
    }

    if metadata_map.is_empty() {
        return Ok(vec![]);
    };

    TransactionMetadata::insert_many(
        metadata_map
            .iter()
            .flat_map(|(tx_id, metadata)| metadata.entries.iter().zip(std::iter::repeat(tx_id)))
            .map(
                |((label, metadata), tx_id)| TransactionMetadataActiveModel {
                    tx_id: Set(*tx_id),
                    label: Set(label.to_le_bytes().to_vec()),
                    payload: Set(metadata.to_cbor_bytes()),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(db_tx)
    .await
}
