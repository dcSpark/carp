use dcspark_core::Regulated;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxAsset {
    #[serde(rename = "aid")]
    pub asset_id: (u64, u64),
    #[serde(rename = "val")]
    pub value: dcspark_core::Value<Regulated>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxOutput {
    #[serde(rename = "addr")]
    pub address: Option<(u64, Option<u64>)>,
    #[serde(rename = "val")]
    pub value: dcspark_core::Value<Regulated>,
    #[serde(rename = "ass")]
    pub assets: Vec<TxAsset>,
}

impl TxOutput {
    pub fn is_banned(&self, banned_addresses: &HashSet<(u64, Option<u64>)>) -> bool {
        self.address
            .map(|address| banned_addresses.contains(&address))
            .unwrap_or(false)
    }

    pub fn is_byron(&self) -> bool {
        self.address.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum TxEvent {
    Full {
        to: Vec<TxOutput>,
        fee: dcspark_core::Value<Regulated>,
        from: Vec<TxOutput>,
    },
    Partial {
        to: Vec<TxOutput>,
    },
}
