use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}

#[derive(Copy, Clone)]
pub enum TxCredentialRelation {
    StakeDelegation,
    StakeRegistration,
    StakeDeregistration,
    Input,
    Output,
}

#[derive(Copy, Clone)]
pub enum AddressCredentialRelation {
    PaymentKey,
    StakeKey,
}

impl From<TxCredentialRelation> for i32 {
    fn from(item: TxCredentialRelation) -> Self {
        match item {
            TxCredentialRelation::StakeDelegation => 0,
            TxCredentialRelation::StakeRegistration => 1,
            TxCredentialRelation::StakeDeregistration => 2,
            TxCredentialRelation::Input => 3,
            TxCredentialRelation::Output => 4,
        }
    }
}

impl From<AddressCredentialRelation> for i32 {
    fn from(item: AddressCredentialRelation) -> Self {
        match item {
            AddressCredentialRelation::PaymentKey => 0,
            AddressCredentialRelation::StakeKey => 1,
        }
    }
}
