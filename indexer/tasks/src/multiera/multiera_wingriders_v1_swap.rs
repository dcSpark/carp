use std::collections::BTreeSet;

use super::multiera_used_inputs::MultieraUsedInputTask;
use super::utils::common::{
    asset_from_pair, get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use super::utils::dex::{
    build_asset, reduce_ada_amount, WR_V1_POOL_FIXED_ADA, WR_V1_POOL_SCRIPT_HASH,
    WR_V1_SWAP_IN_ADA, WR_V1_SWAP_OUT_ADA,
};
use crate::dsl::task_macro::*;
use crate::era_common::get_outputs_for_inputs;
use crate::era_common::OutputWithTxData;
use crate::types::DexSwapDirection;
use crate::{config::EmptyConfig::EmptyConfig, types::AssetPair};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::traverse::{Era, MultiEraOutput};
use pallas::ledger::{
    primitives::ToCanonicalJson,
    traverse::{MultiEraBlock, MultiEraTx},
};

carp_task! {
  name MultieraWingRidersV1SwapTask;
  configuration EmptyConfig;
  doc "Adds WingRiders V1 swaps to the database";
  era multiera;
  dependencies [MultieraUsedInputTask];
  read [multiera_txs, multiera_addresses, multiera_used_inputs_to_outputs_map];
  write [];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.outputs().len() > 0)
  };
  execute |previous_data, task| handle_swap(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_addresses,
      &previous_data.multiera_used_inputs_to_outputs_map,
  );
  merge_result |previous_data, _result| {
  };
}

struct QueuedSwap {
    tx_id: i64,
    address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    asset1: AssetPair,
    asset2: AssetPair,
    amount1: u64,
    amount2: u64,
    direction: DexSwapDirection,
}

async fn handle_swap(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
) -> Result<(), DbErr> {
    // 1) Parse swaps
    let mut queued_swaps = Vec::<QueuedSwap>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            queue_swap(
                &mut queued_swaps,
                tx_body,
                cardano_transaction.id,
                multiera_used_inputs_to_outputs_map,
            );
        }
    }

    if queued_swaps.is_empty() {
        return Ok(());
    }

    // 2) Remove asset duplicates to build a list of all the <policy_id, asset_name> to query for.
    // ADA is ignored, it's not in the NativeAsset DB table
    let mut unique_tokens = BTreeSet::<&(Vec<u8>, Vec<u8>)>::default();
    for p in &queued_swaps {
        if let Some(pair) = &p.asset1 {
            unique_tokens.insert(&pair);
        }
        if let Some(pair) = &p.asset2 {
            unique_tokens.insert(&pair);
        }
    }

    // 3) Query for asset ids
    // TODO use the query result from mean price task?
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
    DexSwap::insert_many(queued_swaps.iter().map(|price| DexSwapActiveModel {
        tx_id: Set(price.tx_id),
        address_id: Set(multiera_addresses[&price.address].model.id),
        asset1_id: Set(asset_pair_to_id_map[&price.asset1]),
        asset2_id: Set(asset_pair_to_id_map[&price.asset2]),
        amount1: Set(price.amount1),
        amount2: Set(price.amount2),
        direction: Set(price.direction.into()),
        ..Default::default()
    }))
    .exec(db_tx)
    .await?;

    Ok(())
}

fn queue_swap(
    queued_swaps: &mut Vec<QueuedSwap>,
    tx: &MultiEraTx,
    tx_id: i64,
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
) {
    // Find the pool address (Note: there should be at most one pool output)
    for pool_output in tx
        .outputs()
        .iter()
        .find(|o| get_sheley_payment_hash(o.address()).as_deref() == Some(WR_V1_POOL_SCRIPT_HASH))
    {
        let redeemers = tx.redeemers().unwrap();

        // Get pool input from redemeers
        let pool_input = redeemers[0].data.to_json()["fields"][0]["int"].as_i64();
        if pool_input.is_none() {
            return;
        }
        let pool_input = pool_input.unwrap();

        // Find main redemeer
        if let Some(redeemer) = redeemers.iter().find(|&r| r.index as i64 == pool_input) {
            let redeemer = redeemer.data.to_json();

            // Extract input list from redemeer
            let redeemer_map: Vec<usize> = redeemer["fields"][2]["list"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["int"].as_i64().unwrap() as usize)
                .collect();
            // Find main transaction
            let parent = redeemer["fields"][0]["int"].as_i64().unwrap() as usize;
            // Restore inputs
            let inputs: Vec<MultiEraOutput> = tx
                .inputs()
                .iter()
                .map(|i| {
                    let output = &multiera_used_inputs_to_outputs_map[&i.hash().to_vec()]
                        [&(i.index() as i64)];
                    MultiEraOutput::decode(output.era, &output.model.payload).unwrap()
                })
                .collect::<Vec<_>>();
            // Zip outputs with redemeer index
            for (output, redeemer) in tx.outputs().iter().skip(1).zip(redeemer_map) {
                // pair input with output
                let input = inputs[redeemer].clone();

                // get information about swap from pool plutus data
                let parent_datum = get_plutus_datum_for_output(&inputs[parent], &tx.plutus_data())
                    .unwrap()
                    .to_json();
                let parse_asset_item = |i, j| {
                    let item = parent_datum["fields"][1]["fields"][0]["fields"][i]["fields"][j]
                        ["bytes"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    hex::decode(item).unwrap()
                };
                let asset1 = build_asset(parse_asset_item(0, 0), parse_asset_item(0, 1));
                let asset2 = build_asset(parse_asset_item(1, 0), parse_asset_item(1, 1));

                // get actual plutus datum
                let input_datum = get_plutus_datum_for_output(&input, &tx.plutus_data())
                    .unwrap()
                    .to_json();
                // identify operation: 0 = swap
                let operation = input_datum["fields"][1]["constructor"].as_i64().unwrap();
                if operation != 0 {
                    tracing::debug!("Operation is not a swap");
                    continue;
                }
                let direction = input_datum["fields"][1]["fields"][0]["constructor"]
                    .as_i64()
                    .unwrap();

                let amount1;
                let amount2;
                if direction == 0 {
                    amount1 = get_asset_amount(&input, &asset1)
                        - reduce_ada_amount(&asset1, WR_V1_SWAP_IN_ADA);
                    amount2 = get_asset_amount(&output, &asset2)
                        - reduce_ada_amount(&asset2, WR_V1_SWAP_OUT_ADA);
                } else {
                    amount1 = get_asset_amount(&output, &asset1)
                        - reduce_ada_amount(&asset1, WR_V1_SWAP_OUT_ADA);
                    amount2 = get_asset_amount(&input, &asset2)
                        - reduce_ada_amount(&asset2, WR_V1_SWAP_IN_ADA);
                }
                queued_swaps.push(QueuedSwap {
                    tx_id,
                    address: pool_output.address().unwrap().to_vec(),
                    asset1,
                    asset2,
                    amount1,
                    amount2,
                    direction: if direction == 0 {
                        DexSwapDirection::SellAsset1
                    } else {
                        DexSwapDirection::BuyAsset1
                    },
                })
            }
        } else {
            tracing::info!("Redeemer not found");
        }
    }
}
