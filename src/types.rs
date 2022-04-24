use serde::Deserialize;

use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}

#[derive(Copy, Clone)]
pub enum TxCredentialRelationValue {
    Witness, // appears in the witness of the tx
    // TODO: differentiate being part of input&output as staking key from payment key
    Input,
    Output,
    StakeDeregistration,
    StakeDelegation,
    StakeRegistration,
    DelegationTarget,
    PoolOwner,
    PoolOperator,
    PoolReward,
    MirRecipient,
    Withdrawal,
    RequiredSigner,
    // TODO: unused input. Ex: collateral input when collateral isn't consumed
    // TODO: native script / mint (in witness)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressCredentialRelationValue {
    PaymentKey,
    StakeKey,
}

impl From<TxCredentialRelationValue> for i32 {
    fn from(item: TxCredentialRelationValue) -> Self {
        match item {
            TxCredentialRelationValue::Witness => 0b1,
            TxCredentialRelationValue::Input => 0b10,
            TxCredentialRelationValue::Output => 0b100,
            TxCredentialRelationValue::StakeDeregistration => 0b1000,
            TxCredentialRelationValue::StakeDelegation => 0b10000,
            TxCredentialRelationValue::StakeRegistration => 0b100000,
            TxCredentialRelationValue::DelegationTarget => 0b1000000,
            TxCredentialRelationValue::PoolOwner => 0b10000000,
            TxCredentialRelationValue::PoolOperator => 0b100000000,
            TxCredentialRelationValue::PoolReward => 0b1000000000,
            TxCredentialRelationValue::MirRecipient => 0b10000000000,
            TxCredentialRelationValue::Withdrawal => 0b100000000000,
            TxCredentialRelationValue::RequiredSigner => 0b1000000000000,
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
