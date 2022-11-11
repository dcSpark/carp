use crate::{
    dsl::task_macro::*,
    multiera::utils::common::{
        asset_from_pair, get_plutus_datum_for_output, get_sheley_payment_hash,
    },
    types::DexSwapDirection,
};
use entity::sea_orm::{DatabaseTransaction, Set};
use pallas::ledger::{
    primitives::alonzo,
    traverse::{MultiEraBlock, MultiEraOutput, MultiEraTx},
};
use std::collections::{BTreeMap, BTreeSet};

use crate::{dsl::database_task::BlockInfo, types::AssetPair};

pub const WR_V1_POOL_SCRIPT_HASH: &str = "e6c90a5923713af5786963dee0fdffd830ca7e0c86a041d9e5833e91";
pub const WR_V1_POOL_FIXED_ADA: u64 = 3_000_000; // every pool UTXO holds this amount of ADA
pub const MS_V1_POOL_SCRIPT_HASH1: &str =
    "e1317b152faac13426e6a83e06ff88a4d62cce3c1634ab0a5ec13309";
pub const MS_V1_POOL_SCRIPT_HASH2: &str =
    "57c8e718c201fba10a9da1748d675b54281d3b1b983c5d1687fc7317";
pub const SS_V1_POOL_SCRIPT_HASH: &str = "4020e7fc2de75a0729c3cc3af715b34d98381e0cdbcfa99c950bc3ac";

pub const WR_V1_SWAP_IN_ADA: u64 = 4_000_000; // oil ADA + agent fee
pub const WR_V1_SWAP_OUT_ADA: u64 = 2_000_000; // oil ADA

pub fn get_pool_output_and_datum<'b>(
    tx: &'b MultiEraTx,
    pool_hashes: &[&str],
) -> Option<(MultiEraOutput<'b>, alonzo::PlutusData)> {
    let pool_hashes = pool_hashes.iter().map(|&s| Some(s)).collect::<Vec<_>>();
    // Note: there should be at most one pool output
    if let Some(output) = tx
        .outputs()
        .iter()
        .find(|o| pool_hashes.contains(&get_sheley_payment_hash(o.address()).as_deref()))
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

pub struct QueuedMeanPrice {
    pub tx_id: i64,
    pub address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    pub asset1: AssetPair,
    pub asset2: AssetPair,
    pub amount1: u64,
    pub amount2: u64,
}

pub struct QueuedSwap {
    pub tx_id: i64,
    pub address: Vec<u8>, // pallas::crypto::hash::Hash<32>
    pub asset1: AssetPair,
    pub asset2: AssetPair,
    pub amount1: u64,
    pub amount2: u64,
    pub direction: DexSwapDirection,
}

pub trait Dex {
    fn queue_mean_price(
        &self,
        queued_prices: &mut Vec<QueuedMeanPrice>,
        tx: &MultiEraTx,
        tx_id: i64,
    );

    fn queue_swap(
        &self,
        queued_swaps: &mut Vec<QueuedSwap>,
        tx: &MultiEraTx,
        tx_id: i64,
        multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    );
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
    ) {
        unimplemented!();
    }

    fn queue_swap(
        &self,
        _queued_swaps: &mut Vec<QueuedSwap>,
        _tx: &MultiEraTx,
        _tx_id: i64,
        _multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    ) {
        unimplemented!();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PoolType {
    WingRidersV1,
    SundaeSwapV1,
    MinSwapV1,
    MinSwapV2,
}

struct PoolConfig {
    pub pool_type: PoolType,
}

impl PoolConfig {
    fn as_trait(&self) -> &dyn Dex {
        match &self.pool_type {
            PoolType::WingRidersV1 => &WingRidersV1 {},
            PoolType::MinSwapV1 => &MinSwapV1 {},
            PoolType::SundaeSwapV1 => &SundaeSwapV1 {},
            _ => &Empty {},
        }
    }
}

pub async fn handle_mean_price(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    multiera_addresses: &BTreeMap<Vec<u8>, AddressInBlock>,
    pool_type: PoolType,
) -> Result<(), DbErr> {
    // 1) Parse mean prices
    let pool = PoolConfig { pool_type };
    let mean_value_trait = pool.as_trait();
    let mut queued_prices = Vec::<QueuedMeanPrice>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            mean_value_trait.queue_mean_price(&mut queued_prices, tx_body, cardano_transaction.id);
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
    pool_type: PoolType,
) -> Result<(), DbErr> {
    // 1) Parse swaps
    let pool = PoolConfig { pool_type };
    let swap_trait = pool.as_trait();
    let mut queued_swaps = Vec::<QueuedSwap>::default();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if cardano_transaction.is_valid {
            swap_trait.queue_swap(
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
