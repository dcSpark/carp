use crate::types::AssetPair;

pub const WR_V1_POOL_SCRIPT_HASH: &str = "e6c90a5923713af5786963dee0fdffd830ca7e0c86a041d9e5833e91";
pub const WR_V1_POOL_FIXED_ADA: u64 = 3_000_000; // every pool UTXO holds this amount of ADA
pub const WR_V1_SWAP_IN_ADA: u64 = 4_000_000; // oil ADA + agent fee
pub const WR_V1_SWAP_OUT_ADA: u64 = 2_000_000; // oil ADA

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
