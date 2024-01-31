use crate::types::TxCredentialRelationValue;
use cml_chain::certs::Credential;
use cml_core::serialization::ToBytes;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RelationMap(pub BTreeMap<i64 /* tx ID in db */, BTreeMap<Vec<u8>, i32>>);

impl RelationMap {
    pub fn for_transaction(&mut self, tx_id: i64) -> &mut BTreeMap<Vec<u8>, i32> {
        self.0.entry(tx_id).or_default()
    }

    pub fn add_relation(
        &mut self,
        tx_id: i64,
        stake_credential: &[u8],
        relation: TxCredentialRelationValue,
    ) {
        let relation_int = i32::from(relation);
        let credential_map = self.for_transaction(tx_id);
        credential_map
            .entry(stake_credential.to_vec())
            .and_modify(|val| *val |= relation_int)
            .or_insert(relation_int);
    }
}
