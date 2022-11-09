use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use super::utils::dex::{
    build_asset, handle_mean_price, Dex, MinSwapV1, PoolType, QueuedMeanPrice,
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

const POOL_SCRIPT_HASH: &str = "e1317b152faac13426e6a83e06ff88a4d62cce3c1634ab0a5ec13309";
const POOL_SCRIPT_HASH2: &str = "57c8e718c201fba10a9da1748d675b54281d3b1b983c5d1687fc7317";

carp_task! {
    name MultieraMinSwapV1MeanPriceTask;
    configuration EmptyConfig;
    doc "Adds Minswap V1 mean price updates to the database";
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
        PoolType::MinSwapV1,
    );
    merge_result |previous_data, _result| {
    };
}

impl Dex for MinSwapV1 {
    fn queue_mean_price(
        &self,
        queued_prices: &mut Vec<QueuedMeanPrice>,
        tx: &MultiEraTx,
        tx_id: i64,
    ) {
        // Find the pool address (Note: there should be at most one pool output)
        for output in tx.outputs().iter().find(|o| {
            get_sheley_payment_hash(o.address()).as_deref() == Some(POOL_SCRIPT_HASH)
                || get_sheley_payment_hash(o.address()).as_deref() == Some(POOL_SCRIPT_HASH2)
        }) {
            // Remark: The datum that corresponds to the pool output's datum hash should be present
            // in tx.plutus_data()
            if let Some(datum) = get_plutus_datum_for_output(output, &tx.plutus_data()) {
                let datum = datum.to_json();

                let get_asset_item = |i, j| {
                    let item = datum["fields"][i]["fields"][j]["bytes"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    hex::decode(item).unwrap()
                };

                let asset1 = build_asset(get_asset_item(0, 0), get_asset_item(0, 1));
                let asset2 = build_asset(get_asset_item(1, 0), get_asset_item(1, 1));

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
