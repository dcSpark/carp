use crate::types::TxCredentialRelationValue;
use pallas::crypto::hash::Hash;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RelationMap(pub BTreeMap<i64 /* tx ID in db */, BTreeMap<Hash<32>, i32>>);

impl RelationMap {
    pub fn bytes_to_pallas(bytes: &[u8]) -> Hash<32> {
        let bytes: [u8; 32] = bytes.try_into().unwrap();
        Hash::<32>::from(bytes)
    }

    pub fn keyhash_to_pallas(
        keyhash: &cardano_multiplatform_lib::crypto::Ed25519KeyHash,
    ) -> Hash<32> {
        RelationMap::bytes_to_pallas(
            &cardano_multiplatform_lib::address::StakeCredential::from_keyhash(keyhash).to_bytes(),
        )
    }

    pub fn scripthash_to_pallas(
        script_hash: &cardano_multiplatform_lib::crypto::ScriptHash,
    ) -> Hash<32> {
        RelationMap::bytes_to_pallas(
            &cardano_multiplatform_lib::address::StakeCredential::from_scripthash(script_hash)
                .to_bytes(),
        )
    }

    pub fn for_transaction(&mut self, tx_id: i64) -> &mut BTreeMap<Hash<32>, i32> {
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
            .entry(RelationMap::bytes_to_pallas(stake_credential))
            .and_modify(|val| *val |= relation_int)
            .or_insert(relation_int);
    }
}
