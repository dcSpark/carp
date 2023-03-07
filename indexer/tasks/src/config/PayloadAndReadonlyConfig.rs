use super::PayloadConfig::PayloadConfig;
use super::ReadonlyConfig::ReadonlyConfig;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct PayloadAndReadonlyConfig {
    pub include_payload: bool,
    pub readonly: bool,
}
