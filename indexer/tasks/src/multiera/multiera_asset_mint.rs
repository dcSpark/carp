use std::collections::{BTreeMap, BTreeSet};

use super::{multiera_txs::MultieraTransactionTask, utils::user_asset::AssetName};
use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::utils::blake2b160;
use cardano_multiplatform_lib::crypto::ScriptHash;
use entity::sea_orm::QueryOrder;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::MultiEraBlock;
use pallas::{
    codec::utils::KeyValuePairs,
    ledger::primitives::alonzo::{self, AuxiliaryData, Metadatum, MetadatumLabel},
};

use crate::dsl::task_macro::*;

carp_task! {
  name MultieraAssetMintTask;
  configuration ReadonlyConfig;
  doc "Adds new tokens and keeps track of mints/burns in general";
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_block, multiera_txs];
  write [multiera_assets];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.mint().len() > 0)
  };
  execute |previous_data, task| handle_mints(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_assets = result;
  };
}

async fn handle_mints(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    readonly: bool,
) -> Result<Vec<NativeAssetModel>, DbErr> {
    let mut queued_mints = Vec::<(i64, (Vec<u8>, Vec<u8>), i64)>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        for (policy_id, assets) in tx_body
            .mint()
            .as_alonzo()
            .iter()
            .map(|x| x.iter())
            .flatten()
        {
            for (asset_name, amount) in assets.iter() {
                queued_mints.push((
                    cardano_transaction.id,
                    (policy_id.to_vec(), asset_name.to_vec()),
                    *amount,
                ));
            }
        }
    }
    if queued_mints.is_empty() {
        return Ok(vec![]);
    }

    if readonly {
        // https://github.com/dcSpark/carp/issues/46
        let mut asset_conditions = Condition::any();
        for (_, (policy_id, asset_name), _) in queued_mints.iter() {
            asset_conditions = asset_conditions.add(
                Condition::all()
                    .add(NativeAssetColumn::PolicyId.eq(policy_id.clone()))
                    .add(NativeAssetColumn::AssetName.eq(asset_name.clone())),
            );
        }

        let assets = NativeAsset::find()
            .filter(asset_conditions)
            .order_by_asc(NativeAssetColumn::Id)
            .all(db_tx)
            .await?;
        return Ok(assets);
    }

    // 1) Remove duplicates to build a list of all the <policy_id, asset_name> pairs that we care about
    // note: duplicates may exist because we're grouping all txs in a block in one batch
    let mut unique_pairs = BTreeMap::<&Vec<u8>, BTreeMap<&Vec<u8>, i64>>::default();
    for (tx_id, pair, _) in &queued_mints {
        unique_pairs
            .entry(&pair.0)
            .and_modify(|asset_names| {
                // we want to keep track of the first tx for each credential
                asset_names.entry(&pair.1).or_insert(*tx_id);
            })
            .or_insert({
                let mut new_set = BTreeMap::<&Vec<u8>, i64>::default();
                new_set.insert(&pair.1, *tx_id);
                new_set
            });
    }

    // 2) Query for which of these pairs already exist in the database
    // https://github.com/dcSpark/carp/issues/46
    let mut mint_conditions = Condition::any();
    for (&asset_name, &policy_id) in unique_pairs
        .iter()
        .flat_map(|(policy_id, assets)| assets.keys().zip(std::iter::repeat(policy_id)))
    {
        mint_conditions = mint_conditions.add(
            Condition::all()
                .add(NativeAssetColumn::PolicyId.eq(policy_id.clone()))
                .add(NativeAssetColumn::AssetName.eq(asset_name.clone())),
        );
    }
    let mut found_assets = NativeAsset::find()
        .filter(mint_conditions)
        .all(db_tx)
        .await?;

    // 3) Find which pairs we need that weren't in the database
    let mut remaining_pairs = unique_pairs.clone();
    for asset in found_assets.iter() {
        let asset_names = remaining_pairs.get_mut(&asset.policy_id).unwrap();
        asset_names.remove(&asset.asset_name);
        if asset_names.is_empty() {
            remaining_pairs.remove(&asset.policy_id);
        }
    }

    // 4) Add the new pairs to the database if there are any
    if !remaining_pairs.is_empty() {
        let mut to_insert = remaining_pairs
            .iter()
            .flat_map(|(policy_id, assets)| assets.iter().zip(std::iter::repeat(policy_id)))
            .map(
                |((&asset_name, tx_id), &policy_id)| NativeAssetActiveModel {
                    policy_id: Set(policy_id.clone()),
                    asset_name: Set(asset_name.clone()),
                    cip14_fingerprint: Set(blake2b160(
                        &[policy_id.as_slice(), asset_name.as_slice()].concat(),
                    )
                    .to_vec()),
                    first_tx: Set(*tx_id),
                    ..Default::default()
                },
            )
            .collect::<Vec<_>>();
        // need to make sure we're inserting addresses in the same order as we added txs
        to_insert.sort_by(|a, b| a.first_tx.as_ref().cmp(b.first_tx.as_ref()));

        found_assets.extend(
            NativeAsset::insert_many(to_insert)
                .exec_many_with_returning(db_tx)
                .await?,
        );
    }

    // 5) Get the list of mints to add
    let mut pair_id_mapping = BTreeMap::<&Vec<u8>, BTreeMap<&Vec<u8>, i64>>::default();
    for entry in &found_assets {
        pair_id_mapping
            .entry(&entry.policy_id)
            .and_modify(|asset_names| {
                asset_names.insert(&entry.asset_name, entry.id);
            })
            .or_insert_with(|| {
                let mut new_set = BTreeMap::<&Vec<u8>, i64>::default();
                new_set.insert(&entry.asset_name, entry.id);
                new_set
            });
    }

    // 6) Add the mint/burn entries
    AssetMint::insert_many(
        queued_mints.iter().map(
            |(tx_id, (policy_id, asset_name), amount)| AssetMintActiveModel {
                tx_id: Set(*tx_id),
                asset_id: Set(pair_id_mapping[policy_id][asset_name]),
                amount: Set(*amount),
            },
        ),
    )
    .exec(db_tx)
    .await?;

    Ok(found_assets)
}
