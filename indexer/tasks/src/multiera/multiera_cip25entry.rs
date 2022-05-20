use std::collections::{BTreeMap, BTreeSet};

use crate::dsl::default_impl::EmptyConfiguration;
use cardano_multiplatform_lib::crypto::ScriptHash;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::{
    codec::utils::KeyValuePairs,
    ledger::primitives::alonzo::{
        self, AuxiliaryData, Metadatum, MetadatumLabel, TransactionBodyComponent,
    },
};

use super::{
    multiera_asset_mint::MultieraAssetMintTask,
    multiera_metadata::MultieraMetadataTask,
    multiera_txs::MultieraTransactionTask,
    utils::{cip25_parse::get_cip25_pairs, user_asset::AssetName},
};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraCip25EntryTask;
  configuration EmptyConfiguration;
  doc "Maps CIP25 entries to the corresponding DB entry for the asset";
  era multiera;
  dependencies [MultieraMetadataTask, MultieraAssetMintTask];
  read [multiera_assets, multiera_metadata];
  write [];
  should_add_task |block, _properties| {
    block.1.auxiliary_data_set.len() > 0
  };
  execute |previous_data, task| handle_entries(
      task.db_tx,
      &previous_data.multiera_metadata,
      &previous_data.multiera_assets,
  );
  merge_result |previous_data, _result| {
  };
}

async fn handle_entries(
    db_tx: &DatabaseTransaction,
    multiera_metadata: &[TransactionMetadataModel],
    multiera_assets: &[NativeAssetModel],
) -> Result<(), DbErr> {
    let mut pair_id_mapping = BTreeMap::<&Vec<u8>, BTreeMap<&Vec<u8>, i64>>::default();
    for entry in multiera_assets.iter() {
        pair_id_mapping
            .entry(&entry.policy_id)
            .and_modify(|asset_names| {
                asset_names.insert(&entry.asset_name, entry.id);
            })
            .or_insert({
                let mut new_set = BTreeMap::<&Vec<u8>, i64>::default();
                new_set.insert(&entry.asset_name, entry.id);
                new_set
            });
    }

    let mut to_insert: Vec<Cip25EntryActiveModel> = vec![];
    for metadata in multiera_metadata {
        if let Ok(pairs) =
            &get_cip25_pairs(&Metadatum::decode_fragment(metadata.payload.as_slice()).unwrap())
        {
            for (policy_id, asset_name) in pairs
                .iter()
                .flat_map(|(policy_id, assets)| assets.iter().zip(std::iter::repeat(policy_id)))
            {
                if let Some(native_asset_id) = pair_id_mapping
                    .get(&asset_name)
                    .and_then(|assets| assets.get(policy_id))
                {
                    to_insert.push(Cip25EntryActiveModel {
                        tx_id: Set(metadata.tx_id),
                        label: Set(metadata.label.clone()),
                        native_asset_id: Set(*native_asset_id),
                    });
                }
            }
        }
    }

    if !to_insert.is_empty() {
        Cip25Entry::insert_many(to_insert).exec(db_tx).await?;
    }
    Ok(())
}
