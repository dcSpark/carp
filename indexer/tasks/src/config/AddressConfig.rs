use pallas::ledger::addresses::Address;
use pallas::ledger::primitives::alonzo::PlutusScript;
use pallas::ledger::primitives::babbage::PlutusV2Script;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AddressConfig {
    pub address: String,
}
