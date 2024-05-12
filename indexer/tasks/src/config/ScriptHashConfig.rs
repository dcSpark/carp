#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ScriptHashConfig {
    pub script_hash: String, // hex-encoded
}
