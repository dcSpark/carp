#[derive(Copy, Clone)]
pub enum TxCredentialRelationValue {
    Witness,     // appears in the witness of the tx
    UnusedInput, // collateral input when collateral isn't consumed or opposite if collateral was consumed
    UnusedInputStake,
    Input,
    ReferenceInput,
    Output,
    InputStake,          // occurs as the staking key of an input
    ReferenceInputStake, // occurs as the staking key of an input
    OutputStake,         // occurs as the staking key of an output
    UnusedOutput, // collateral output when collateral isn't consumed or opposite if collateral was consumed
    UnusedOutputStake,
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
    InNativeScript, // keyhash in scripts including mints
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressCredentialRelationValue {
    PaymentKey,
    StakeKey,
}

// Note: keep in sync with the Javascript type RelationFilterType
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
            TxCredentialRelationValue::InNativeScript => 0b10000000000000,
            TxCredentialRelationValue::UnusedInput => 0b100000000000000,
            TxCredentialRelationValue::UnusedInputStake => 0b1000000000000000,
            TxCredentialRelationValue::InputStake => 0b10000000000000000,
            TxCredentialRelationValue::OutputStake => 0b100000000000000000,
            TxCredentialRelationValue::UnusedOutput => 0b1000000000000000000,
            TxCredentialRelationValue::UnusedOutputStake => 0b10000000000000000000,
            TxCredentialRelationValue::ReferenceInput => 0b100000000000000000000,
            TxCredentialRelationValue::ReferenceInputStake => 0b1000000000000000000000,
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
