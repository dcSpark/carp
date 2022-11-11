use std::collections::BTreeMap;

use pallas::ledger::{primitives::ToCanonicalJson, traverse::MultiEraTx};

use crate::{era_common::OutputWithTxData, multiera::utils::common::get_asset_amount};

use super::common::{
    build_asset, get_pool_output_and_datum, Dex, QueuedMeanPrice, QueuedSwap, SundaeSwapV1,
    SS_V1_POOL_SCRIPT_HASH,
};

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

    fn queue_swap(
        &self,
        _queued_swaps: &mut Vec<QueuedSwap>,
        _tx: &MultiEraTx,
        _tx_id: i64,
        _multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    ) {
        unimplemented!()
    }
}
