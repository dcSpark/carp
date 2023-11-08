use std::collections::BTreeMap;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::database_task::BlockGlobalInfo;
use entity::sea_orm::QueryOrder;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraMeta};
use pallas::{
    codec::utils::KeyValuePairs,
    ledger::primitives::alonzo::{self, AuxiliaryData, Metadatum, MetadatumLabel},
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
    block.1.has_aux_data()
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

    let mut metadata_map = BTreeMap::<i64 /* id */, MultiEraMeta>::default();

    let txs = block.1.txs();

    for (idx, tx) in txs.iter().enumerate() {
        let tx_id = &multiera_txs[idx].id;
        let meta = tx.metadata();

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
            .flat_map(|(tx_id, metadata)| {
                metadata
                    .as_alonzo()
                    .unwrap()
                    .iter()
                    .zip(std::iter::repeat(tx_id))
            })
            .map(
                |((label, metadata), tx_id)| TransactionMetadataActiveModel {
                    tx_id: Set(*tx_id),
                    label: Set(label.to_le_bytes().to_vec()),
                    payload: Set(metadata.encode_fragment().unwrap()),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(db_tx)
    .await
}
