use std::collections::BTreeSet;

use entity::{
    prelude::*,
    sea_orm::{entity::*, prelude::*, Condition, DatabaseTransaction},
};
use pallas::{
    codec::utils::KeepRaw,
    crypto::hash::Hasher,
    ledger::{
        addresses,
        primitives::{alonzo, babbage::DatumOption},
        traverse::{MultiEraOutput, Subject},
    },
};

use crate::types::AssetPair;

pub fn get_shelley_payment_hash(
    address: Result<addresses::Address, addresses::Error>,
) -> Option<String> {
    if let Ok(addresses::Address::Shelley(shelley_address)) = address {
        Some(hex::encode(shelley_address.payment().as_hash()))
    } else {
        None
    }
}

pub fn get_asset_amount(output: &MultiEraOutput, pair: &AssetPair) -> u64 {
    output
        .assets()
        .iter()
        .filter(|asset| match &asset.subject {
            Subject::Lovelace => pair.is_none(),
            Subject::NativeAsset(policy_id, asset_name) => {
                pair == &Some((policy_id.to_vec(), asset_name.to_vec()))
            }
        })
        .map(|a| a.quantity)
        .sum()
}

pub fn get_plutus_datum_for_output(
    output: &MultiEraOutput,
    plutus_data: &[&KeepRaw<alonzo::PlutusData>],
) -> Option<alonzo::PlutusData> {
    match output.datum() {
        Some(DatumOption::Data(datum)) => Some(datum.0),
        Some(DatumOption::Hash(hash)) => plutus_data
            .iter()
            .find(|datum| Hasher::<256>::hash_cbor(datum) == hash)
            .map(|&d| d.clone().unwrap()),
        None => None,
    }
}

pub async fn asset_from_pair(
    db_tx: &DatabaseTransaction,
    pairs: &[(Vec<u8> /* policy id */, Vec<u8> /* asset name */)],
) -> Result<Vec<NativeAssetModel>, DbErr> {
    // https://github.com/dcSpark/carp/issues/46
    let mut asset_conditions = Condition::any();
    for (policy_id, asset_name) in pairs.iter() {
        asset_conditions = asset_conditions.add(
            Condition::all()
                .add(NativeAssetColumn::PolicyId.eq(policy_id.clone()))
                .add(NativeAssetColumn::AssetName.eq(asset_name.clone())),
        );
    }

    let assets = NativeAsset::find()
        .filter(asset_conditions)
        .all(db_tx)
        .await?;
    Ok(assets)
}
