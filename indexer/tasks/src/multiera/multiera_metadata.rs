use std::{
    collections::BTreeMap,
};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::{
    codec::utils::KeyValuePairs,
    ledger::primitives::alonzo::{self, AuxiliaryData, Metadatum, MetadatumLabel},
};

use super::multiera_txs::MultieraTransactionTask;

use crate::{database_task::PrerunResult, task_macro::*};

#[derive(Copy, Clone)]
pub struct MultieraMetadataPrerunData();

carp_task! {
  name MultieraMetadataTask;
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_txs];
  write [multiera_metadata];
  should_add_task |_block, _properties| -> MultieraMetadataPrerunData {
    PrerunResult::RunTaskWith(MultieraMetadataPrerunData())
  };
  execute |previous_data, task| handle_metadata(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_metadata = result;
  };
}

async fn handle_metadata(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
) -> Result<Vec<TransactionMetadataModel>, DbErr> {
    let mut metadata_map =
        BTreeMap::<i64 /* id */, &KeyValuePairs<MetadatumLabel, Metadatum>>::default();
    for (idx, data) in block.1.auxiliary_data_set.iter() {
        let tx_id = &multiera_txs[*idx as usize].id;
        let opt_entries = match data {
            AuxiliaryData::Shelley(metadata) => Some(metadata),
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                ..
            } => Some(transaction_metadata),
            AuxiliaryData::Alonzo(data) => data.metadata.as_ref(),
        };

        opt_entries.and_then(|entries| {
            if !entries.is_empty() {
                // it's possible for metadata to just be an empty list (no labels)
                // ex: tx hash 3fd58bb02af554c0653be693386525b521ca586cbeb6b2e2cc782ab9a1041708
                metadata_map.insert(*tx_id, entries)
            } else {
                None
            }
        });
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
                },
            ),
    )
    .exec_many_with_returning(db_tx)
    .await?)
}
