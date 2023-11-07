use cml_chain::crypto::ScriptHash;

pub type AssetName = Vec<u8>;
pub type PolicyId = Vec<u8>;
pub type Payload = Vec<u8>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cip25ParseError(pub String);

impl std::error::Error for Cip25ParseError {}

impl std::fmt::Display for Cip25ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
