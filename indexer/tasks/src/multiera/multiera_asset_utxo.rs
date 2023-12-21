use super::multiera_stake_credentials::MultieraStakeCredentialTask;
use crate::config::AddressConfig::PayloadAndReadonlyConfig;
use crate::config::EmptyConfig::EmptyConfig;
use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::task_macro::*;
use crate::multiera::dex::common::filter_outputs_and_datums_by_address;
use crate::multiera::multiera_txs::MultieraTransactionTask;
use crate::multiera::multiera_used_inputs::MultieraUsedInputTask;
use crate::multiera::multiera_used_outputs::MultieraOutputTask;
use crate::types::AddressCredentialRelationValue;
use cardano_multiplatform_lib::error::DeserializeError;
use cml_core::serialization::{FromBytes, ToBytes};
use cml_crypto::RawBytesEncoding;
use entity::sea_orm::Condition;
use entity::transaction_output::Model;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::babbage::DatumOption;
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{Asset, MultiEraInput, MultiEraOutput};
use projected_nft_sdk::{Owner, State, Status};
use sea_orm::{FromQueryResult, JoinType, QuerySelect, QueryTrait};
use std::collections::{BTreeSet, HashMap};

carp_task! {
  name MultieraAssetUtxos;
  configuration EmptyConfig;
  doc "Parses utxo movements for native assets";
  era multiera;
  dependencies [MultieraUsedInputTask, MultieraOutputTask];
  read [multiera_txs, multiera_outputs, multiera_used_inputs_to_outputs_map];
  write [];
  should_add_task |block, _properties| {
    !block.1.is_empty()
  };
  execute |previous_data, task| handle(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_outputs,
      &previous_data.multiera_used_inputs_to_outputs_map,
  );
  merge_result |previous_data, _result| {
  };
}

async fn handle(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_outputs: &[TransactionOutputModel],
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
) -> Result<(), DbErr> {
    let mut queued_inserts = vec![];

    // this stores the result before searching for the asset ids in the table
    struct PartialEntry {
        utxo_id: i64,
        amount: Option<i64>,
        tx_id: i64,
        // policy_id + asset_name
        asset: (Vec<u8>, Vec<u8>),
    }

    let mut condition = Condition::any();

    let outputs_map: HashMap<_, _> = multiera_outputs
        .iter()
        .map(|output| ((output.tx_id, output.output_index), output))
        .collect();

    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        for input in tx_body.inputs().iter().chain(tx_body.collateral().iter()) {
            let utxo = multiera_used_inputs_to_outputs_map
                .get(input.hash().as_ref())
                .and_then(|by_index| by_index.get(&(input.index() as i64)));

            let utxo = if let Some(utxo) = utxo {
                utxo
            } else {
                // this can happen if the tx was not valid, in which case the
                // input is not spent.
                continue;
            };

            let output = MultiEraOutput::decode(utxo.era, &utxo.model.payload).unwrap();

            for asset in output.non_ada_assets() {
                let (policy_id, asset_name, value) = match asset {
                    Asset::Ada(_) => continue,
                    Asset::NativeAsset(policy_id, asset_name, value) => {
                        (policy_id, asset_name, value)
                    }
                };

                if value == 0 {
                    continue;
                }

                condition = condition.add(
                    Condition::all()
                        .add(entity::native_asset::Column::PolicyId.eq(policy_id.as_ref()))
                        .add(entity::native_asset::Column::AssetName.eq(asset_name.clone())),
                );

                queued_inserts.push(PartialEntry {
                    utxo_id: utxo.model.id,
                    amount: None,
                    tx_id: cardano_transaction.id,
                    asset: (policy_id.as_ref().to_vec(), asset_name),
                });
            }
        }

        for (output_index, output) in tx_body
            .outputs()
            .iter()
            .chain(tx_body.collateral_return().iter())
            .enumerate()
        {
            let address = output
                .address()
                .map_err(|err| DbErr::Custom(format!("invalid pallas address: {}", err)))?
                .to_vec();

            let address = cardano_multiplatform_lib::address::Address::from_bytes(address)
                .map_err(|err| DbErr::Custom(format!("cml can't parse address: {}", err)))?;

            if address.payment_cred().is_none() {
                continue;
            };

            let output_model = match outputs_map.get(&(cardano_transaction.id, output_index as i32))
            {
                None => {
                    continue;
                }
                Some(output) => output,
            };

            for asset in output.non_ada_assets() {
                let (policy_id, asset_name, value) = match asset {
                    Asset::Ada(_) => continue,
                    Asset::NativeAsset(policy_id, asset_name, value) => {
                        (policy_id, asset_name, value)
                    }
                };

                if value == 0 {
                    continue;
                }

                condition = condition.add(
                    Condition::all()
                        .add(entity::native_asset::Column::PolicyId.eq(policy_id.as_ref()))
                        .add(entity::native_asset::Column::AssetName.eq(asset_name.clone())),
                );

                queued_inserts.push(PartialEntry {
                    utxo_id: output_model.id,
                    amount: Some(value as i64),
                    tx_id: cardano_transaction.id,
                    asset: (policy_id.as_ref().to_vec(), asset_name),
                });
            }
        }
    }

    if !queued_inserts.is_empty() {
        let asset_map = entity::native_asset::Entity::find()
            .filter(condition)
            .all(db_tx)
            .await?
            .into_iter()
            .map(|asset| ((asset.policy_id, asset.asset_name), asset.id))
            .collect::<HashMap<_, _>>();

        AssetUtxo::insert_many(
            queued_inserts
                .into_iter()
                .map(|partial_entry| {
                    let asset_id = asset_map.get(&partial_entry.asset).ok_or_else(|| {
                        DbErr::Custom(format!(
                            "Asset not found: {}-{}",
                            hex::encode(&partial_entry.asset.0),
                            String::from_utf8_lossy(&partial_entry.asset.1)
                        ))
                    })?;

                    Ok(entity::asset_utxos::ActiveModel {
                        asset_id: Set(*asset_id),
                        utxo_id: Set(partial_entry.utxo_id),
                        amount: Set(partial_entry.amount),
                        tx_id: Set(partial_entry.tx_id),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, DbErr>>()?,
        )
        .exec(db_tx)
        .await?;
    }

    Ok(())
}
