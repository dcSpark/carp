// Note: this is taken from https://github.com/txpipe/oura/blob/393d47484cf87423e6f54e224c656f236159085d/src/mapper/cip25.rs
// We should instead at some point have a proper cip25_rs

use std::collections::{BTreeMap, BTreeSet};

use cml_chain::auxdata::metadata::TransactionMetadatum;
use cml_chain::crypto::ScriptHash;
use cml_core::serialization::{Serialize, ToBytes};

use super::user_asset::{AssetName, Cip25ParseError, Payload, PolicyId};

// Heuristic approach for filtering policy entries. This is the best I could
// come up with. Is there a better, official way?
fn is_policy_key(key: &TransactionMetadatum) -> Option<PolicyId> {
    match key {
        TransactionMetadatum::Bytes { bytes, .. } if bytes.len() == 28 => Some(bytes.clone()),
        TransactionMetadatum::Text { text, .. } if text.len() == 56 => hex::decode(text).ok(),
        _ => None,
    }
}

// Heuristic approach for filtering asset entries. Even less strict than when
// searching for policies. In this case, we only check for valid data types.
// There's probably a much more formal approach.
fn is_asset_key(key: &TransactionMetadatum) -> Option<AssetName> {
    match key {
        TransactionMetadatum::Bytes { bytes, .. } if bytes.len() <= 32 => Some(bytes.clone()),
        TransactionMetadatum::Text { text, .. } if text.as_bytes().len() <= 32 => {
            Some(text.as_bytes().to_vec())
        }
        _ => None,
    }
}

fn search_cip25_version(content_721: &TransactionMetadatum) -> Option<String> {
    match content_721 {
        TransactionMetadatum::Map(entries) => {
            entries.entries.iter().find_map(|(key, value)| match key {
                TransactionMetadatum::Text { text, .. } if text == "version" => match value {
                    TransactionMetadatum::Text { text, .. } => Some(text.clone()),
                    _ => None,
                },
                _ => None,
            })
        }
        _ => None,
    }
}

fn get_cip25_assets(
    _version: &str,
    content: &TransactionMetadatum,
) -> Result<BTreeMap<AssetName, Payload>, Cip25ParseError> {
    let mut result = BTreeMap::<AssetName, Payload>::default();
    if let TransactionMetadatum::Map(entries) = content {
        for (key, sub_content) in entries.entries.iter() {
            if let Some(asset) = &is_asset_key(key) {
                result.insert(asset.clone(), sub_content.to_cbor_bytes());
            }
        }
    } else {
        return Err(Cip25ParseError(
            "invalid metadatum type for policy inside 721 label".to_string(),
        ));
    }

    Ok(result)
}

#[allow(clippy::type_complexity)]
pub fn get_cip25_pairs(
    content: &TransactionMetadatum,
) -> Result<(String, BTreeMap<PolicyId, BTreeMap<AssetName, Payload>>), Cip25ParseError> {
    let version = search_cip25_version(content).unwrap_or_else(|| "1.0".to_string());

    let mut result = BTreeMap::<PolicyId, BTreeMap<AssetName, Payload>>::default();
    if let TransactionMetadatum::Map(entries) = content {
        for (key, sub_content) in entries.entries.iter() {
            if let Some(policy_id) = is_policy_key(key) {
                if let Ok(asset_names) = get_cip25_assets(&version, sub_content) {
                    result.insert(policy_id, asset_names);
                }
            }
        }
    } else {
        return Err(Cip25ParseError(
            "invalid metadatum type for 721 label".to_string(),
        ));
    }

    Ok((version, result))
}
