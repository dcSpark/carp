use cml_chain::certs::Credential;
use cml_chain::transaction::DatumOption;
use cml_chain::NonemptySetPlutusData;
use cml_core::serialization::{Deserialize, Serialize, ToBytes};
use cml_crypto::RawBytesEncoding;
use cml_multi_era::utils::{MultiEraTransactionInput, MultiEraTransactionOutput};
use std::collections::BTreeSet;

use crate::era_common::OutputWithTxData;
use entity::block::EraValue;
use entity::{
    prelude::*,
    sea_orm::{entity::*, prelude::*, Condition, DatabaseTransaction},
};

use crate::types::AssetPair;
use crate::utils::blake2b256;

pub fn get_shelley_payment_hash(address: cml_chain::address::Address) -> Option<String> {
    let payment = match address {
        cml_chain::address::Address::Base(address) => address.payment,
        // idk whether we should parse it here or not
        cml_chain::address::Address::Ptr(address) => address.payment,
        cml_chain::address::Address::Enterprise(address) => address.payment,
        // reward address is a staking address
        cml_chain::address::Address::Reward(_) => return None,
        cml_chain::address::Address::Byron(_) => return None,
    };

    match payment {
        Credential::PubKey { hash, .. } => Some(hash.to_hex()),
        Credential::Script { hash, .. } => Some(hash.to_hex()),
    }
}

pub fn get_asset_amount(
    output: &cml_multi_era::utils::MultiEraTransactionOutput,
    pair: &AssetPair,
) -> u64 {
    match pair {
        None => output.amount().coin,
        Some((pair_policy_id, pair_asset_name)) => output
            .amount()
            .multiasset
            .iter()
            .flat_map(|(policy_id, assets)| {
                assets
                    .iter()
                    .map(|(asset_name, value)| (*policy_id, asset_name, value))
            })
            .filter(|(policy_id, asset_name, _value)| {
                policy_id.to_raw_bytes() == pair_policy_id && asset_name.get() == pair_asset_name
            })
            .map(|(_policy_id, _asset_name, value)| value)
            .sum(),
    }
}

pub fn get_plutus_datum_for_output(
    output: &cml_multi_era::utils::MultiEraTransactionOutput,
    plutus_data: &Option<NonemptySetPlutusData>,
) -> Option<cml_chain::plutus::PlutusData> {
    let output = match output {
        MultiEraTransactionOutput::Byron(_) => {
            return None;
        }
        MultiEraTransactionOutput::Shelley(output) => output,
    };

    let datum_option = match output.datum() {
        Some(datum) => datum,
        None => {
            return None;
        }
    };

    match datum_option {
        DatumOption::Datum { datum, .. } => Some(datum),
        DatumOption::Hash { datum_hash, .. } => plutus_data.as_ref().and_then(|non_empty_set| {
            non_empty_set
                .iter()
                .find(|datum| datum.hash() == datum_hash)
                .cloned()
        }),
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

pub fn output_from_bytes(utxo: &OutputWithTxData) -> Result<MultiEraTransactionOutput, DbErr> {
    let output = match utxo.era {
        EraValue::Byron => MultiEraTransactionOutput::Byron(
            cml_chain::byron::ByronTxOut::from_cbor_bytes(&utxo.model.payload).map_err(|err| {
                DbErr::Custom(format!("can't decode byron output payload: {err}"))
            })?,
        ),
        _ => MultiEraTransactionOutput::Shelley(
            cml_chain::transaction::TransactionOutput::from_cbor_bytes(&utxo.model.payload)
                .map_err(|err| {
                    DbErr::Custom(format!("can't decode shelley output payload: {err}"))
                })?,
        ),
    };

    Ok(output)
}
