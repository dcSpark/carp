// Note: this is taken from https://github.com/txpipe/oura/blob/393d47484cf87423e6f54e224c656f236159085d/src/mapper/cip25.rs
// We should instead at some point have a proper cip25_rs

use std::collections::{BTreeMap, BTreeSet};

use cardano_multiplatform_lib::crypto::ScriptHash;
use pallas::ledger::primitives::alonzo::Metadatum;

use super::user_asset::{AssetName, Cip25ParseError, PolicyId};

// Heuristic approach for filtering policy entries. This is the best I could
// come up with. Is there a better, official way?
fn is_policy_key(key: &Metadatum) -> Option<ScriptHash> {
    match key {
        Metadatum::Bytes(x) if x.len() == 28 => Some(ScriptHash::from_bytes(x.to_vec()).unwrap()),
        Metadatum::Text(x) if x.len() == 56 => Some(ScriptHash::from_hex(x).unwrap()),
        _ => None,
    }
}

// Heuristic approach for filtering asset entries. Even less strict than when
// searching for policies. In this case, we only check for valid data types.
// There's probably a much more formal approach.
fn is_asset_key(key: &Metadatum) -> Option<AssetName> {
    match key {
        Metadatum::Bytes(x) => Some(AssetName::try_from(x.as_slice()).unwrap()),
        Metadatum::Text(x) => Some(AssetName::try_from(x.as_bytes()).unwrap()),
        _ => None,
    }
}

fn search_cip25_version(content_721: &Metadatum) -> Option<String> {
    match content_721 {
        Metadatum::Map(entries) => entries.iter().find_map(|(key, value)| match key {
            Metadatum::Text(x) if x == "version" => match value {
                Metadatum::Text(value) => Some(value.to_owned()),
                _ => None,
            },
            _ => None,
        }),
        _ => None,
    }
}

fn get_cip25_assets(
    _version: &str,
    content: &Metadatum,
) -> Result<BTreeSet<AssetName>, Cip25ParseError> {
    let mut result = BTreeSet::<AssetName>::default();
    if let Metadatum::Map(entries) = content {
        for (key, _) in entries.iter() {
            if let Some(asset) = &is_asset_key(key) {
                result.insert(asset.clone());
            }
        }
    } else {
        return Err(Cip25ParseError(
            "invalid metadatum type for policy inside 721 label".to_string(),
        ));
    }

    Ok(result)
}

pub fn get_cip25_pairs(
    content: &Metadatum,
) -> Result<BTreeMap<Vec<u8>, BTreeSet<Vec<u8>>>, Cip25ParseError> {
    let version = search_cip25_version(content).unwrap_or_else(|| "1.0".to_string());

    let mut result = BTreeMap::<Vec<u8>, BTreeSet<Vec<u8>>>::default();
    if let Metadatum::Map(entries) = content {
        for (key, sub_content) in entries.iter() {
            if let Some(policy_id) = is_policy_key(key) {
                if let Ok(asset_names) = get_cip25_assets(&version, sub_content) {
                    result.insert(policy_id.to_bytes(), asset_names);
                }
            }
        }
    } else {
        return Err(Cip25ParseError(
            "invalid metadatum type for 721 label".to_string(),
        ));
    }

    Ok(result)
}
