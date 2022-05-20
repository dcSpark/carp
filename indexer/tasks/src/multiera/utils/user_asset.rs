use cardano_multiplatform_lib::crypto::ScriptHash;

pub type AssetName = Vec<u8>;
pub type PolicyId = ScriptHash;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cip25ParseError(pub String);

impl std::error::Error for Cip25ParseError {}

impl std::fmt::Display for Cip25ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
