use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use super::utils::dex::{
    build_asset, get_pool_output_and_datum, handle_mean_price, Dex, PoolType, QueuedMeanPrice,
    SundaeSwapV1, SS_V1_POOL_SCRIPT_HASH,
};
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::dsl::task_macro::*;
use crate::{config::EmptyConfig::EmptyConfig, types::AssetPair};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::{
    primitives::ToCanonicalJson,
    traverse::{MultiEraBlock, MultiEraTx},
};
use std::collections::BTreeSet;

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
    fn queue_mean_price(
        &self,
        queued_prices: &mut Vec<QueuedMeanPrice>,
        tx: &MultiEraTx,
        tx_id: i64,
    ) {
        if let Some((output, datum)) = get_pool_output_and_datum(tx, &vec![SS_V1_POOL_SCRIPT_HASH])
        {
            let datum = datum.to_json();

            let get_asset_item = |i, j| {
                let item = datum["fields"][0]["fields"][i]["fields"][j]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string();
                hex::decode(item).unwrap()
            };

            let asset1 = build_asset(get_asset_item(0, 0), get_asset_item(0, 1));
            let asset2 = build_asset(get_asset_item(1, 0), get_asset_item(1, 1));

            let amount1 = get_asset_amount(&output, &asset1);
            let amount2 = get_asset_amount(&output, &asset2);

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
