#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct PayloadConfig {
    pub include_payload: bool,
}
