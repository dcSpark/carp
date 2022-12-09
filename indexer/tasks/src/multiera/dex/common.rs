use crate::{dsl::database_task::BlockInfo, types::AssetPair};
use crate::{
    dsl::task_macro::*,
    multiera::utils::common::{
        asset_from_pair, get_plutus_datum_for_output, get_shelley_payment_hash,
    },
};
use entity::dex_swap::Operation;
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::{
    codec::utils::KeepRaw,
    ledger::{
        primitives::alonzo,
        traverse::{MultiEraBlock, MultiEraOutput, MultiEraTx},
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// Returns an output and it's datum only if the output's payment hash is in `payment_hashes`
/// and the plutus datum is known.
pub fn filter_outputs_and_datums_by_hash<'b>(
    outputs: &[MultiEraOutput<'b>],
    payment_hashes: &[&str],
    plutus_data: &[&KeepRaw<alonzo::PlutusData>],
) -> Vec<(MultiEraOutput<'b>, alonzo::PlutusData)> {
    let payment_hashes = payment_hashes.iter().map(|&s| Some(s)).collect::<Vec<_>>();
    outputs
        .iter()
        .filter_map(|o| {
            if payment_hashes.contains(&get_shelley_payment_hash(o.address()).as_deref()) {
                get_plutus_datum_for_output(o, plutus_data).map(|datum| (o.clone(), datum))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Returns an output and it's datum only if the output's address is in `addresses`
/// and the plutus datum is known.
pub fn filter_outputs_and_datums_by_address<'b>(
    outputs: &[MultiEraOutput<'b>],
    addresses: &[&str],
    plutus_data: &[&KeepRaw<alonzo::PlutusData>],
) -> Vec<(MultiEraOutput<'b>, alonzo::PlutusData)> {
    let addresses = addresses.iter().map(|&s| Some(s)).collect::<Vec<_>>();
    outputs
        .iter()
        .filter_map(|o| {
            if addresses.contains(&o.address().ok().and_then(|a| a.to_bech32().ok()).as_deref()) {
                get_plutus_datum_for_output(o, plutus_data).map(|datum| (o.clone(), datum))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub struct QueuedMeanPrice {
    pub tx_id: i64,
    pub address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    pub dex_type: DexType,
    pub asset1: AssetPair,
    pub asset2: AssetPair,
    pub amount1: u64,
    pub amount2: u64,
}

pub struct QueuedSwap {
    pub tx_id: i64,
    pub address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    pub dex_type: DexType,
    pub asset1: AssetPair,
    pub asset2: AssetPair,
    pub amount1: u64,
    pub amount2: u64,
    pub operation: Operation,
}

pub trait Dex {
    /// Handle the rest of the assets on the pool address
    fn queue_mean_price(
        &self,
        queued_prices: &mut Vec<QueuedMeanPrice>,
        tx: &MultiEraTx,
        tx_id: i64,
    ) -> Result<(), String>;

    /// Handle amount of each swap operation
    fn queue_swap(
        &self,
        queued_swaps: &mut Vec<QueuedSwap>,
        tx: &MultiEraTx,
        tx_id: i64,
        multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    ) -> Result<(), String>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct WingRidersV1;
#[derive(Debug, PartialEq, Eq)]
pub struct MinSwapV1;
#[derive(Debug, PartialEq, Eq)]
pub struct SundaeSwapV1;
#[derive(Debug, PartialEq, Eq)]
pub struct Empty;

impl Dex for Empty {
    fn queue_mean_price(
        &self,
        _queued_prices: &mut Vec<QueuedMeanPrice>,
        _tx: &MultiEraTx,
        _tx_id: i64,
    ) -> Result<(), String> {
        unimplemented!();
    }

    fn queue_swap(
        &self,
        _queued_swaps: &mut Vec<QueuedSwap>,
        _tx: &MultiEraTx,
        _tx_id: i64,
        _multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    ) -> Result<(), String> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DexType {
    WingRidersV1,
    SundaeSwapV1,
    MinSwapV1,
    MinSwapV2,
}

impl From<DexType> for i32 {
    fn from(item: DexType) -> Self {
        match item {
            DexType::WingRidersV1 => 0,
            DexType::SundaeSwapV1 => 1,
            DexType::MinSwapV1 => 2,
            DexType::MinSwapV2 => 3,
        }
    }
}

impl DexType {
    fn as_trait(&self) -> &dyn Dex {
        match &self {
            DexType::WingRidersV1 => &WingRidersV1 {},
            DexType::MinSwapV1 => &MinSwapV1 {},
            DexType::SundaeSwapV1 => &SundaeSwapV1 {},
            _ => &Empty {},
        }
    }
}

pub async fn handle_mean_price(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
    pool_type: DexType,
) -> Result<(), DbErr> {
    // 1) Parse mean prices
    let pool = pool_type;
    let mean_value_trait = pool.as_trait();
    let mut queued_prices = Vec::<QueuedMeanPrice>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            let result = mean_value_trait.queue_mean_price(
                &mut queued_prices,
                tx_body,
                cardano_transaction.id,
            );
            if result.is_err() {
                tracing::warn!(
                    "Failed to parse mean price for tx {}: {}",
                    cardano_transaction.id,
                    result.err().unwrap(),
                );
            }
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
            unique_tokens.insert(pair);
        }
        if let Some(pair) = &p.asset2 {
            unique_tokens.insert(pair);
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
    DexSwap::insert_many(
        queued_prices
            .iter()
            .filter(|price| {
                // In the unlikely case that an asset is not in the DB, skip this price update
                asset_pair_to_id_map.contains_key(&price.asset1)
                    && asset_pair_to_id_map.contains_key(&price.asset2)
            })
            .map(|price| DexSwapActiveModel {
                tx_id: Set(price.tx_id),
                address_id: Set(multiera_addresses[&price.address].model.id),
                dex: Set(i32::from(price.dex_type.clone())),
                asset1_id: Set(asset_pair_to_id_map[&price.asset1]),
                asset2_id: Set(asset_pair_to_id_map[&price.asset2]),
                amount1: Set(price.amount1),
                amount2: Set(price.amount2),
                operation: Set(Operation::Mean.into()),
                ..Default::default()
            }),
    )
    .exec(db_tx)
    .await?;

    Ok(())
}

pub fn build_asset(policy_id: Vec<u8>, asset_name: Vec<u8>) -> AssetPair {
    if policy_id.is_empty() && asset_name.is_empty() {
        None
    } else {
        Some((policy_id, asset_name))
    }
}

pub fn reduce_ada_amount(pair: &AssetPair, amount: u64) -> u64 {
    if pair.is_none() {
        amount
    } else {
        0
    }
}

pub async fn handle_swap(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    dex_type: DexType,
) -> Result<(), DbErr> {
    // 1) Parse swaps
    let swap_trait = dex_type.as_trait();
    let mut queued_swaps = Vec::<QueuedSwap>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            let result = swap_trait.queue_swap(
                &mut queued_swaps,
                tx_body,
                cardano_transaction.id,
                multiera_used_inputs_to_outputs_map,
            );
            if result.is_err() {
                tracing::warn!(
                    "Failed to parse swaps for tx {}: {}",
                    cardano_transaction.id,
                    result.err().unwrap()
                );
            }
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
            unique_tokens.insert(pair);
        }
        if let Some(pair) = &p.asset2 {
            unique_tokens.insert(pair);
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
    DexSwap::insert_many(
        queued_swaps
            .iter()
            .filter(|price| {
                // In the unlikely case that an asset is not in the DB, skip this price update
                asset_pair_to_id_map.contains_key(&price.asset1)
                    && asset_pair_to_id_map.contains_key(&price.asset2)
            })
            .map(|price| DexSwapActiveModel {
                tx_id: Set(price.tx_id),
                address_id: Set(multiera_addresses[&price.address].model.id),
                dex: Set(price.dex_type.clone().into()),
                asset1_id: Set(asset_pair_to_id_map[&price.asset1]),
                asset2_id: Set(asset_pair_to_id_map[&price.asset2]),
                amount1: Set(price.amount1),
                amount2: Set(price.amount2),
                operation: Set(price.operation.into()),
                ..Default::default()
            }),
    )
    .exec(db_tx)
    .await?;

    Ok(())
}
