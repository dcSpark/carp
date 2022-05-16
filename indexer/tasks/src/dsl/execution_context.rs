pub use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
pub use std::collections::BTreeMap;

#[macro_export]
macro_rules! data_to_type {
  // genesis
  (genesis_txs) => { Vec<TransactionModel> };
  (genesis_addresses) => { Vec<AddressModel> };
  (genesis_outputs) => { Vec<TransactionOutputModel> };

  // byron
  (byron_txs) => { Vec<TransactionModel> };
  (byron_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };
  (byron_inputs) => { Vec<TransactionInputModel> };
  (byron_outputs) => { Vec<TransactionOutputModel> };

  // multiera
  (multiera_txs) => { Vec<TransactionModel> };
  (vkey_relation_map) => { RelationMap };
  (multiera_queued_addresses_relations) => { BTreeSet<QueuedAddressCredentialRelation> };
  (multiera_stake_credential) => { BTreeMap<Vec<u8>, StakeCredentialModel> };
  (multiera_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };
  (multiera_metadata) => { Vec<TransactionMetadataModel> };
  (multiera_outputs) => { Vec<TransactionOutputModel> };
  (multiera_used_inputs) => { Vec<TransactionInputModel> };
}

pub(crate) use data_to_type;
