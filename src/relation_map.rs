use crate::types::TxCredentialRelationValue;
use pallas::crypto::hash::Hash;
use std::collections::BTreeMap;

pub struct RelationMapValue {
    pub credential_id: i64,
    pub relation: i32,
}
#[derive(Default)]
pub struct RelationMap(pub BTreeMap<Hash<28>, RelationMapValue>);

impl RelationMap {
    pub fn bytes_to_pallas(bytes: &Vec<u8>) -> Hash<28> {
        let bytes: [u8; 28] = bytes.clone().try_into().unwrap();
        Hash::<28>::from(bytes)
    }

    pub fn keyhash_to_pallas(
        keyhash: &cardano_multiplatform_lib::crypto::Ed25519KeyHash,
    ) -> Hash<28> {
        RelationMap::bytes_to_pallas(&keyhash.to_bytes())
    }

    pub fn scripthash_to_pallas(
        script_hash: &cardano_multiplatform_lib::crypto::ScriptHash,
    ) -> Hash<28> {
        RelationMap::bytes_to_pallas(&script_hash.to_bytes())
    }

    pub fn add_relation(
        &mut self,
        stake_credential: &entity::stake_credential::Model,
        relation: TxCredentialRelationValue,
    ) -> () {
        self.0
            .entry(RelationMap::bytes_to_pallas(&stake_credential.credential))
            .and_modify(|val| val.relation |= i32::from(relation))
            .or_insert(RelationMapValue {
                credential_id: stake_credential.id,
                relation: relation.into(),
            });
    }
}
