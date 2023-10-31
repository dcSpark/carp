use cardano_multiplatform_lib::error::DeserializeError;
use cml_core::serialization::FromBytes;
use pallas::ledger::primitives::babbage::DatumOption;
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{Asset, MultiEraOutput};
use projected_nft_sdk::{State, Status};
use std::collections::{BTreeSet, HashMap};

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::types::AddressCredentialRelationValue;
use entity::sea_orm::Condition;
use entity::transaction_output::Model;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};

use crate::dsl::task_macro::*;

use super::multiera_stake_credentials::MultieraStakeCredentialTask;

use crate::config::AddressConfig::PayloadAndReadonlyConfig;
use crate::multiera::multiera_txs::MultieraTransactionTask;
use crate::multiera::multiera_used_inputs::MultieraUsedInputTask;
use crate::multiera::multiera_used_outputs::MultieraOutputTask;

carp_task! {
  name MultiEraProjectedNftTask;
  configuration PayloadAndReadonlyConfig;
  doc "Parses projected NFT contract data";
  era multiera;
  dependencies [MultieraOutputTask, MultieraUsedInputTask];
  read [multiera_txs, multiera_outputs, multiera_used_inputs_to_outputs_map];
  write [];
  should_add_task |block, _properties| {
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_projected_nft(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_outputs,
      &previous_data.multiera_used_inputs_to_outputs_map,
      task.config.address.clone(),
  );
  merge_result |previous_data, _result| {
  };
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectedNftOperation {
    Lock,
    Unlocking,
    Claim,
    ParseError,
    NoDatum,
    NotInlineDatum,
}

impl From<ProjectedNftOperation> for i32 {
    fn from(item: ProjectedNftOperation) -> Self {
        match item {
            ProjectedNftOperation::Lock => 0,
            ProjectedNftOperation::Unlocking => 1,
            ProjectedNftOperation::Claim => 2,
            ProjectedNftOperation::ParseError => 3,
            ProjectedNftOperation::NoDatum => 4,
            ProjectedNftOperation::NotInlineDatum => 5,
        }
    }
}

impl TryFrom<i32> for ProjectedNftOperation {
    type Error = String;

    fn try_from(item: i32) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(ProjectedNftOperation::Lock),
            1 => Ok(ProjectedNftOperation::Unlocking),
            2 => Ok(ProjectedNftOperation::Claim),
            3 => Ok(ProjectedNftOperation::ParseError),
            4 => Ok(ProjectedNftOperation::NoDatum),
            5 => Ok(ProjectedNftOperation::NotInlineDatum),
            _ => Err("can't parse projeced nft operation".to_string()),
        }
    }
}

async fn handle_projected_nft(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_outputs: &[TransactionOutputModel],
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    address: String,
) -> Result<(), DbErr> {
    let config_address = hex::decode(address).map_err(|err| {
        DbErr::Custom(format!(
            "can't decode projected nft config address hex: {:?}",
            err
        ))
    })?;
    let config_address = cardano_multiplatform_lib::address::Address::from_bytes(config_address)
        .map_err(|err| DbErr::Custom(format!("cml can't parse config address: {:?}", err)))?;
    let config_payment_cred = match config_address.payment_cred() {
        None => {
            return Err(DbErr::Custom(
                "provided projected nft config address contains no payment cred".to_string(),
            ))
        }
        Some(pk) => pk,
    };

    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        let mut outputs_map = HashMap::new();
        for output_model in multiera_outputs
            .iter()
            .filter(|output| output.tx_id == cardano_transaction.id)
        {
            outputs_map.insert(output_model.output_index, output_model.clone());
        }

        let mut potential_projected_nft_inputs = vec![];
        for input in tx_body.inputs() {
            let output_for_input = multiera_used_inputs_to_outputs_map
                .get(&input.hash().to_vec())
                .ok_or(DbErr::Custom(format!(
                    "can't find input: {}@{}",
                    input.hash().clone(),
                    input.index()
                )))?
                .get(&(input.index() as i64))
                .ok_or(DbErr::Custom(format!(
                    "can't find input: {}@{}",
                    input.hash().clone(),
                    input.index()
                )))?
                .clone();
            potential_projected_nft_inputs.push(output_for_input.model.id);
        }

        let seen_projected_nfts = ProjectedNft::find()
            .filter(
                potential_projected_nft_inputs
                    .into_iter()
                    .fold(Condition::any(), |cond, addition| {
                        cond.add(ProjectedNftColumn::UtxoId.eq(addition))
                    }),
            )
            .all(db_tx)
            .await?;

        let mut queued_projected_nft_records = vec![];

        for projected_nft in seen_projected_nfts.iter().filter(|projected_nft| {
            projected_nft.operation == i32::from(ProjectedNftOperation::Unlocking)
        }) {
            queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                utxo_id: Set(None),
                tx_id: Set(cardano_transaction.id),
                asset: Set(projected_nft.asset.clone()),
                amount: Set(projected_nft.amount),
                operation: Set(ProjectedNftOperation::Claim.into()),
                plutus_datum: Set(vec![]),
                ..Default::default()
            })
        }

        for (output_index, output) in tx_body.outputs().iter().enumerate() {
            let address = output
                .address()
                .map_err(|err| DbErr::Custom(format!("invalid pallas address: {}", err)))?
                .to_vec();
            let address = cardano_multiplatform_lib::address::Address::from_bytes(address)
                .map_err(|err| DbErr::Custom(format!("cml can't parse address: {}", err)))?;
            let output_payment_cred = match address.payment_cred() {
                None => continue,
                Some(pk) => pk,
            };

            if output_payment_cred != config_payment_cred {
                continue;
            }

            let output_model = match outputs_map.get(&(output_index as i32)) {
                None => {
                    return Err(DbErr::RecordNotFound(format!(
                        "can't find output with index {output_index} of tx {}",
                        cardano_transaction.id
                    )))
                }
                Some(output) => output.clone(),
            };

            let (operation, plutus_data) = extract_operation_and_datum(output);

            for asset in output.non_ada_assets() {
                queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                    utxo_id: Set(Some(output_model.id)),
                    tx_id: Set(cardano_transaction.id),
                    asset: Set(asset.subject()),
                    amount: Set(match asset {
                        Asset::Ada(value) => value as i64,
                        Asset::NativeAsset(_, _, value) => value as i64,
                    }),
                    operation: Set(operation.into()),
                    plutus_datum: Set(plutus_data.clone()),
                    ..Default::default()
                });
            }
        }

        ProjectedNft::insert_many(queued_projected_nft_records)
            .exec(db_tx)
            .await?;
    }

    Ok(())
}

fn extract_operation_and_datum(output: &MultiEraOutput) -> (ProjectedNftOperation, Vec<u8>) {
    let datum_option = match output.datum() {
        Some(datum) => DatumOption::from(datum.clone()),
        None => {
            return (ProjectedNftOperation::NoDatum, vec![]);
        }
    };

    let datum = match datum_option {
        DatumOption::Hash(hash) => {
            return (ProjectedNftOperation::NotInlineDatum, hash.to_vec());
        }
        DatumOption::Data(datum) => datum.0.encode_fragment().unwrap(),
    };

    let parsed = match cml_chain::plutus::PlutusData::from_bytes(datum.clone()) {
        Ok(parsed) => parsed,
        Err(_) => return (ProjectedNftOperation::ParseError, datum),
    };

    let parsed = match projected_nft_sdk::State::try_from(parsed) {
        Ok(parsed) => parsed,
        Err(_) => return (ProjectedNftOperation::ParseError, datum),
    };

    match parsed.status {
        Status::Locked => (ProjectedNftOperation::Lock, datum),
        Status::Unlocking { .. } => (ProjectedNftOperation::Unlocking, datum),
    }
}
