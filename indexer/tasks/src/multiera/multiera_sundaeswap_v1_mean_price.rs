use std::collections::BTreeSet;
use super::utils::dex::{handle_mean_price, QueuedMeanPrice, Dex, SundaeSwapV1, PoolType};
use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::dsl::task_macro::*;
use crate::{config::EmptyConfig::EmptyConfig, types::AssetPair};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::{
    primitives::ToCanonicalJson,
    traverse::{MultiEraBlock, MultiEraTx},
};

const POOL_SCRIPT_HASH: &str = "4020e7fc2de75a0729c3cc3af715b34d98381e0cdbcfa99c950bc3ac";

carp_task! {
  name MultieraSundaeSwapV1MeanPriceTask;
  configuration EmptyConfig;
  doc "Adds SundaeSwap V1 mean price updates to the database";
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
      PoolType::SundaeSwapV1,
  );
  merge_result |previous_data, _result| {
  };
}

impl Dex for SundaeSwapV1 {
    fn queue_mean_price(&self, queued_prices: &mut Vec<QueuedMeanPrice>, tx: &MultiEraTx, tx_id: i64) {
        // Find the pool address (Note: there should be at most one pool output)
        for output in tx
            .outputs()
            .iter()
            .find(|o| get_sheley_payment_hash(o.address()).as_deref() == Some(POOL_SCRIPT_HASH))
        {
            // Remark: The datum that corresponds to the pool output's datum hash should be present
            // in tx.plutus_data()
            if let Some(datum) = get_plutus_datum_for_output(output, &tx.plutus_data()) {
                let datum = datum.to_json();

                let get_asset_item = |i, j| {
                    let item = datum["fields"][0]["fields"][i]["fields"][j]["bytes"]
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

                let amount1 = get_asset_amount(output, &asset1);
                let amount2 = get_asset_amount(output, &asset2);

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