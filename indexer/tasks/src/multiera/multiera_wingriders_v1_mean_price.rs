use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use super::utils::dex::{
    WR_V1_POOL_FIXED_ADA, WR_V1_POOL_SCRIPT_HASH,
    build_asset, handle_mean_price, reduce_ada_amount, Dex, PoolType, QueuedMeanPrice,
    WingRidersV1,
};
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::dsl::task_macro::*;
use crate::{config::EmptyConfig::EmptyConfig, types::AssetPair};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::primitives::alonzo;
use pallas::ledger::{
    primitives::ToCanonicalJson,
    traverse::{MultiEraBlock, MultiEraOutput, MultiEraTx},
};
use std::collections::BTreeSet;

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
      PoolType::WingRidersV1,
  );
  merge_result |previous_data, _result| {
  };
}

impl Dex for WingRidersV1 {
    fn queue_mean_price(
        &self,
        queued_prices: &mut Vec<QueuedMeanPrice>,
        tx: &MultiEraTx,
        tx_id: i64,
    ) {
        // Find the pool address (Note: there should be at most one pool output)
        for output in tx
            .outputs()
            .iter()
            .find(|o| get_sheley_payment_hash(o.address()).as_deref() == Some(WR_V1_POOL_SCRIPT_HASH))
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
                let asset1 = build_asset(get_asset_item(0, 0), get_asset_item(0, 1));
                let asset2 = build_asset(get_asset_item(1, 0), get_asset_item(1, 1));

                let get_fixed_ada = |pair: &AssetPair| -> u64 {
                    if pair.is_none() {
                        WR_V1_POOL_FIXED_ADA
                    } else {
                        0
                    }
                };
                let amount1 = get_asset_amount(output, &asset1)
                    - treasury1
                    - reduce_ada_amount(&asset1, WR_V1_POOL_FIXED_ADA);
                let amount2 = get_asset_amount(output, &asset2)
                    - treasury2
                    - reduce_ada_amount(&asset2, WR_V1_POOL_FIXED_ADA);

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
}

pub fn get_pool_output<'b>(tx: &'b MultiEraTx) -> Option<(MultiEraOutput<'b>, alonzo::PlutusData)> {
    // Note: there should be at most one pool output
    if let Some(output) = tx
        .outputs()
        .iter()
        .find(|o| get_sheley_payment_hash(o.address()).as_deref() == Some(WR_V1_POOL_SCRIPT_HASH))
    {
        // Remark: The datum that corresponds to the pool output's datum hash should be present
        // in tx.plutus_data()
        if let Some(datum) = get_plutus_datum_for_output(output, &tx.plutus_data()) {
            Some((output.clone(), datum))
        } else {
            None
        }
    } else {
        None
    }
}

fn queue_mean_price(queued_prices: &mut Vec<QueuedMeanPrice>, tx: &MultiEraTx, tx_id: i64) {
    if let Some((output, datum)) = get_pool_output(tx) {
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
        let asset1 = build_asset(get_asset_item(0, 0), get_asset_item(0, 1));
        let asset2 = build_asset(get_asset_item(1, 0), get_asset_item(1, 1));

        let amount1 = get_asset_amount(&output, &asset1)
            - treasury1
            - reduce_ada_amount(&asset1, WR_V1_POOL_FIXED_ADA);
        let amount2 = get_asset_amount(&output, &asset2)
            - treasury2
            - reduce_ada_amount(&asset2, WR_V1_POOL_FIXED_ADA);

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