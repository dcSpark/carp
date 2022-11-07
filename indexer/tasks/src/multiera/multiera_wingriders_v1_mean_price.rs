use std::collections::BTreeSet;

use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_shelley_payment_hash,
};
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::dsl::task_macro::*;
use crate::{config::EmptyConfig::EmptyConfig, types::AssetPair};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::{
    primitives::ToCanonicalJson,
    traverse::{MultiEraBlock, MultiEraTx},
};

const POOL_SCRIPT_HASH: &str = "e6c90a5923713af5786963dee0fdffd830ca7e0c86a041d9e5833e91";
const POOL_FIXED_ADA: u64 = 3_000_000; // every pool UTXO holds this amount of ADA

carp_task! {
  name MultieraWingRidersV1MeanPriceTask;
  configuration EmptyConfig;
  doc "Adds WingRiders V1 mean price updates to the database";
  era multiera;
  dependencies [MultieraAddressTask];
  read [multiera_txs, multiera_addresses];
  write [];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.outputs().len() > 0)
  };
  execute |previous_data, task| handle_mean_price(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_addresses,
  );
  merge_result |previous_data, _result| {
  };
}

struct QueuedMeanPrice {
    tx_id: i64,
    address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    asset1: AssetPair,
    asset2: AssetPair,
    amount1: u64,
    amount2: u64,
}

async fn handle_mean_price(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
) -> Result<(), DbErr> {
    // 1) Parse mean prices
    let mut queued_prices = Vec::<QueuedMeanPrice>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            queue_mean_price(&mut queued_prices, tx_body, cardano_transaction.id);
        }
    }

    if queued_prices.is_empty() {
        return Ok(());
    }

    // 2) Remove asset duplicates to build a list of all the <policy_id, asset_name> to query for.
    // ADA is ignored, it's not in the NativeAsset DB table
    let mut unique_tokens = BTreeSet::<&(Vec<u8>, Vec<u8>)>::default();
    for p in &queued_prices {
        if let Some(pair) = &p.asset1 {
            unique_tokens.insert(&pair);
        }
        if let Some(pair) = &p.asset2 {
            unique_tokens.insert(&pair);
        }
    }

    // 3) Query for asset ids
    let found_assets = asset_from_pair(
        db_tx,
        &unique_tokens
            .iter()
            .map(|(policy_id, asset_name)| (policy_id.clone(), asset_name.clone()))
            .collect::<Vec<_>>(),
    )
    .await?;
    let mut asset_pair_to_id_map = found_assets
        .into_iter()
        .map(|asset| (Some((asset.policy_id, asset.asset_name)), Some(asset.id)))
        .collect::<BTreeMap<_, _>>();
    asset_pair_to_id_map.insert(None, None); // ADA

    // 4) Add mean prices to DB
    DexMeanPrice::insert_many(queued_prices.iter().map(|price| DexMeanPriceActiveModel {
        tx_id: Set(price.tx_id),
        address_id: Set(multiera_addresses[&price.address].model.id),
        asset1_id: Set(asset_pair_to_id_map[&price.asset1]),
        asset2_id: Set(asset_pair_to_id_map[&price.asset2]),
        amount1: Set(price.amount1),
        amount2: Set(price.amount2),
        ..Default::default()
    }))
    .exec(db_tx)
    .await?;

    Ok(())
}

fn queue_mean_price(queued_prices: &mut Vec<QueuedMeanPrice>, tx: &MultiEraTx, tx_id: i64) {
    // Find the pool address (Note: there should be at most one pool output)
    for output in tx
        .outputs()
        .iter()
        .find(|o| get_shelley_payment_hash(o.address()).as_deref() == Some(POOL_SCRIPT_HASH))
    {
        // Remark: The datum that corresponds to the pool output's datum hash should be present
        // in tx.plutus_data()
        if let Some(datum) = get_plutus_datum_for_output(output, &tx.plutus_data()) {
            let datum = datum.to_json();

            let treasury1 = datum["fields"][1]["fields"][2]["int"].as_u64().unwrap();
            let treasury2 = datum["fields"][1]["fields"][3]["int"].as_u64().unwrap();

            let get_asset_item = |i, j| {
                let item = datum["fields"][1]["fields"][0]["fields"][i]["fields"][j]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string();
                hex::decode(item).unwrap()
            };
            let get_asset = |policy_id: Vec<u8>, asset_name: Vec<u8>| {
                if policy_id.is_empty() && asset_name.is_empty() {
                    None
                } else {
                    Some((policy_id, asset_name))
                }
            };
            let asset1 = get_asset(get_asset_item(0, 0), get_asset_item(0, 1));
            let asset2 = get_asset(get_asset_item(1, 0), get_asset_item(1, 1));

            let get_fixed_ada = |pair: &AssetPair| -> u64 {
                if pair.is_none() {
                    POOL_FIXED_ADA
                } else {
                    0
                }
            };
            let amount1 = get_asset_amount(output, &asset1) - treasury1 - get_fixed_ada(&asset1);
            let amount2 = get_asset_amount(output, &asset2) - treasury2 - get_fixed_ada(&asset2);

            queued_prices.push(QueuedMeanPrice {
                tx_id,
                address: output.address().unwrap().to_vec(),
                asset1,
                asset2,
                amount1,
                amount2,
            });
        }
    }
}
