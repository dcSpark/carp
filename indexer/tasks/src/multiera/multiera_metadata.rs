use std::collections::BTreeMap;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use entity::sea_orm::QueryOrder;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::MultiEraBlock;
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
    block.1.auxiliary_data_set.len() > 0
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
    block: BlockInfo<'_, MultiEraBlock<'_>>,
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

    let mut metadata_map =
        BTreeMap::<i64 /* id */, &KeyValuePairs<MetadatumLabel, Metadatum>>::default();

    for (idx, tx) in block.1.txs().iter().enumerate() {
        let tx_id = &multiera_txs[*idx as usize].id;
        tx.metadata()
            .iter()
            // it's possible for metadata to just be an empty list (no labels)
            // ex: tx hash 3fd58bb02af554c0653be693386525b521ca586cbeb6b2e2cc782ab9a1041708
            .filter(|x| !x.entries().is_empty())
            .for_each(|x| metadata_map.insert(*tx_id, x.entries()));
    }

    if metadata_map.is_empty() {
        return Ok(vec![]);
    };

    Ok(TransactionMetadata::insert_many(
        metadata_map
            .iter()
            .flat_map(|(tx_id, metadata)| metadata.iter().zip(std::iter::repeat(tx_id)))
            .map(
                |((label, metadata), tx_id)| TransactionMetadataActiveModel {
                    tx_id: Set(*tx_id),
                    label: Set(u64::from(label).to_le_bytes().to_vec()),
                    payload: Set(metadata.encode_fragment().unwrap()),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(db_tx)
    .await?)
}
