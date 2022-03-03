use serde::Deserialize;

use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}

#[derive(Copy, Clone)]
pub enum TxCredentialRelationValue {
    StakeDelegation,
    StakeRegistration,
    StakeDeregistration,
    Input,
    Output,
}

#[derive(Copy, Clone)]
pub enum AddressCredentialRelationValue {
    PaymentKey,
    StakeKey,
}

impl From<TxCredentialRelationValue> for i32 {
    fn from(item: TxCredentialRelationValue) -> Self {
        match item {
            TxCredentialRelationValue::StakeDelegation => 0,
            TxCredentialRelationValue::StakeRegistration => 1,
            TxCredentialRelationValue::StakeDeregistration => 2,
            TxCredentialRelationValue::Input => 3,
            TxCredentialRelationValue::Output => 4,
        }
    }
}

impl From<AddressCredentialRelationValue> for i32 {
    fn from(item: AddressCredentialRelationValue) -> Self {
        match item {
            AddressCredentialRelationValue::PaymentKey => 0,
            AddressCredentialRelationValue::StakeKey => 1,
        }
    }
}

pub type GenesisFile = Vec<GenesisData>;

#[derive(Debug, Deserialize)]
pub struct GenesisData {
    pub hash: String,
    pub index: u64,
    pub address: String,
}
