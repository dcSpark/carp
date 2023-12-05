use anyhow::anyhow;
use cardano_multiplatform_lib::error::DeserializeError;
use cml_core::serialization::FromBytes;
use cml_crypto::RawBytesEncoding;
use pallas::ledger::primitives::alonzo::{Redeemer, RedeemerTag};
use pallas::ledger::primitives::babbage::DatumOption;
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{Asset, MultiEraOutput, MultiEraTx};
use projected_nft_sdk::{Owner, Redeem, State, Status};
use sea_orm::{FromQueryResult, JoinType, QuerySelect, QueryTrait};
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
use crate::multiera::dex::common::filter_outputs_and_datums_by_address;
use crate::multiera::multiera_txs::MultieraTransactionTask;
use crate::multiera::multiera_used_inputs::MultieraUsedInputTask;
use crate::multiera::multiera_used_outputs::MultieraOutputTask;

carp_task! {
  name MultiEraProjectedNftTask;
  configuration PayloadAndReadonlyConfig;
  doc "Parses projected NFT contract data";
  era multiera;
  dependencies [MultieraUsedInputTask, MultieraOutputTask];
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum ProjectedNftOperation {
    Lock,
    Unlocking,
    Claim,
    #[default]
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

#[derive(FromQueryResult, Debug, Clone)]
pub(crate) struct ProjectedNftInputsQueryOutputResult {
    pub id: i64,
    pub tx_id: i64,
    pub output_index: i32,
    pub tx_hash: Vec<u8>,
    pub operation: i32,
    pub owner_address: Vec<u8>,
    pub asset: String,
    pub amount: i64,
}

async fn handle_projected_nft(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_outputs: &[TransactionOutputModel],
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    address: String,
) -> Result<(), DbErr> {
    let config_payment_cred = get_payment_cred(address)?;

    // spent projected nfts in current transaction
    let used_projected_nfts =
        get_projected_nft_inputs(db_tx, multiera_used_inputs_to_outputs_map).await?;

    let mut queued_projected_nft_records = vec![];

    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        let redeemers = tx_body
            .redeemers()
            .map(get_projected_nft_redeemers)
            .unwrap_or(Ok(BTreeMap::new()))?;

        let _partial_withdrawals = handle_claims_and_partial_withdraws(
            tx_body,
            cardano_transaction,
            &redeemers,
            &used_projected_nfts,
            &mut queued_projected_nft_records,
        );

        let outputs_map = get_output_index_to_outputs_map(cardano_transaction, multiera_outputs);

        let mut scheduled_projected_nft_outputs = vec![];

        for (output_index, output) in tx_body.outputs().iter().enumerate() {
            let address = output
                .address()
                .map_err(|err| DbErr::Custom(format!("invalid pallas address: {}", err)))?
                .to_hex();

            let output_payment_cred = get_payment_cred(address)?;

            if output_payment_cred != config_payment_cred {
                continue;
            }

            let output_model = outputs_map
                .get(&(output_index as i32))
                .ok_or(DbErr::RecordNotFound(format!(
                    "can't find output with index {output_index} of tx {}",
                    cardano_transaction.id
                )))?
                .clone();

            let projected_nft_data = extract_operation_and_datum(output);

            for asset in output.non_ada_assets() {
                scheduled_projected_nft_outputs.push(entity::projected_nft::ActiveModel {
                    owner_address: Set(projected_nft_data.address.clone()),
                    previous_utxo_tx_output_index: Set(
                        projected_nft_data.previous_utxo_tx_output_index
                    ),
                    previous_utxo_tx_hash: Set(projected_nft_data.previous_utxo_tx_hash.clone()),
                    hololocker_utxo_id: Set(Some(output_model.id)),
                    tx_id: Set(cardano_transaction.id),
                    asset: Set(asset.subject()),
                    amount: Set(match asset {
                        Asset::Ada(value) => value as i64,
                        Asset::NativeAsset(_, _, value) => value as i64,
                    }),
                    operation: Set(projected_nft_data.operation.into()),
                    plutus_datum: Set(projected_nft_data.plutus_data.clone()),
                    for_how_long: Set(projected_nft_data.for_how_long),
                    ..Default::default()
                });
            }
        }

        queued_projected_nft_records.append(&mut scheduled_projected_nft_outputs);
    }

    if !queued_projected_nft_records.is_empty() {
        ProjectedNft::insert_many(queued_projected_nft_records)
            .exec(db_tx)
            .await?;
    }

    Ok(())
}

fn get_payment_cred(
    address: String,
) -> Result<cardano_multiplatform_lib::address::StakeCredential, DbErr> {
    let config_address = hex::decode(address).map_err(|err| {
        DbErr::Custom(format!(
            "can't decode projected nft config address hex: {:?}",
            err
        ))
    })?;

    let config_address = cardano_multiplatform_lib::address::Address::from_bytes(config_address)
        .map_err(|err| DbErr::Custom(format!("cml can't parse config address: {:?}", err)))?;
    match config_address.payment_cred() {
        None => Err(DbErr::Custom(
            "provided projected nft config address contains no payment cred".to_string(),
        )),
        Some(pk) => Ok(pk),
    }
}

async fn get_projected_nft_inputs(
    db_tx: &DatabaseTransaction,
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
) -> Result<BTreeMap<Vec<u8>, BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>>, DbErr> {
    let inputs_condition = multiera_used_inputs_to_outputs_map
        .iter()
        .flat_map(|(_input_tx_id, map)| {
            map.iter().map(|(_output_index, data)| {
                (data.model.tx_id, data.model.output_index, data.model.id) // input and utxo id
            })
        })
        .fold(Condition::any(), |cond, (tx_id, _output_index, utxo_id)| {
            cond.add(
                Condition::all()
                    .add(ProjectedNftColumn::HololockerUtxoId.eq(utxo_id))
                    .add(ProjectedNftColumn::TxId.eq(tx_id)),
            )
        });

    let projected_nfts = ProjectedNft::find()
        .select_only()
        .column(TransactionOutputColumn::Id)
        .column(TransactionOutputColumn::TxId)
        .column(TransactionOutputColumn::OutputIndex)
        .column(ProjectedNftColumn::Operation)
        .column(ProjectedNftColumn::Asset)
        .column(ProjectedNftColumn::Amount)
        .column(ProjectedNftColumn::OwnerAddress)
        .column_as(TransactionColumn::Hash, "tx_hash")
        .join(
            JoinType::InnerJoin,
            ProjectedNftRelation::TransactionOutput.def(),
        )
        .join(JoinType::InnerJoin, ProjectedNftRelation::Transaction.def())
        .filter(inputs_condition)
        .into_model::<ProjectedNftInputsQueryOutputResult>()
        .all(db_tx)
        .await?;

    let mut result: BTreeMap<Vec<u8>, BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>> =
        BTreeMap::new();
    for nft in projected_nfts {
        result
            .entry(nft.tx_hash.clone())
            .or_default()
            .entry(nft.output_index as i64)
            .or_default()
            .push(nft);
    }
    Ok(result)
}

fn handle_claims_and_partial_withdraws(
    tx_body: &MultiEraTx,
    cardano_transaction: &TransactionModel,
    redeemers: &BTreeMap<i64, Redeem>,
    used_projected_nfts: &BTreeMap<
        Vec<u8>,
        BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>,
    >,
    queued_projected_nft_records: &mut Vec<ProjectedNftActiveModel>,
) -> Vec<ProjectedNftInputsQueryOutputResult> {
    let mut partially_withdrawn = Vec::new();

    for (input_index, input) in tx_body.inputs().iter().enumerate() {
        let entry = if let Some(entry) = used_projected_nfts.get(&input.hash().to_vec()) {
            entry
        } else {
            continue;
        };

        let projected_nfts = if let Some(projected_nfts) = entry.get(&(input.index() as i64)) {
            projected_nfts
        } else {
            continue;
        };

        for projected_nft in projected_nfts {
            if projected_nft.operation == i32::from(ProjectedNftOperation::Unlocking) {
                queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                    hololocker_utxo_id: Set(None),
                    tx_id: Set(cardano_transaction.id),
                    asset: Set(projected_nft.asset.clone()),
                    amount: Set(projected_nft.amount),
                    operation: Set(ProjectedNftOperation::Claim.into()),
                    plutus_datum: Set(vec![]),
                    owner_address: Set(projected_nft.owner_address.clone()),
                    previous_utxo_tx_hash: Set(projected_nft.tx_hash.clone()),
                    previous_utxo_tx_output_index: Set(Some(projected_nft.output_index as i64)),
                    for_how_long: Set(None),
                    ..Default::default()
                });
            }
            if projected_nft.operation == i32::from(ProjectedNftOperation::Lock) {
                let redeemer = match redeemers.get(&(input_index as i64)) {
                    None => {
                        tracing::warn!(
                            "No redeemer found for {}, {}",
                            hex::encode(cardano_transaction.hash.clone()),
                            input_index
                        );
                        continue;
                    }
                    Some(redeem) => redeem,
                };

                if redeemer.partial_withdraw {
                    partially_withdrawn.push(projected_nft.clone());
                }
            }
        }
    }

    partially_withdrawn
}

fn get_output_index_to_outputs_map(
    cardano_transaction: &TransactionModel,
    multiera_outputs: &[TransactionOutputModel],
) -> HashMap<i32, TransactionOutputModel> {
    let mut outputs_map = HashMap::new();
    for output_model in multiera_outputs
        .iter()
        .filter(|output| output.tx_id == cardano_transaction.id)
    {
        outputs_map.insert(output_model.output_index, output_model.clone());
    }

    outputs_map
}

#[derive(Debug, Clone, Default)]
struct ProjectedNftData {
    pub previous_utxo_tx_hash: Vec<u8>,
    pub previous_utxo_tx_output_index: Option<i64>,
    pub address: Vec<u8>,
    pub plutus_data: Vec<u8>,
    pub operation: ProjectedNftOperation,
    pub for_how_long: Option<i64>,
}

fn extract_operation_and_datum(output: &MultiEraOutput) -> ProjectedNftData {
    let datum_option = match output.datum() {
        Some(datum) => DatumOption::from(datum.clone()),
        None => {
            return ProjectedNftData {
                operation: ProjectedNftOperation::NoDatum,
                ..Default::default()
            };
        }
    };

    let datum = match datum_option {
        DatumOption::Hash(hash) => {
            return ProjectedNftData {
                plutus_data: hash.to_vec(),
                // the contract expects inline datums only
                operation: ProjectedNftOperation::NotInlineDatum,
                ..Default::default()
            };
        }
        DatumOption::Data(datum) => datum.0.encode_fragment().unwrap(),
    };

    let parsed = match cml_chain::plutus::PlutusData::from_bytes(datum.clone()) {
        Ok(parsed) => parsed,
        Err(_) => {
            return ProjectedNftData {
                plutus_data: datum,
                operation: ProjectedNftOperation::ParseError,
                ..Default::default()
            }
        }
    };

    let parsed = match projected_nft_sdk::State::try_from(parsed) {
        Ok(parsed) => parsed,
        Err(_) => {
            return ProjectedNftData {
                plutus_data: datum,
                operation: ProjectedNftOperation::ParseError,
                ..Default::default()
            }
        }
    };

    let owner_address = match parsed.owner {
        Owner::PKH(pkh) => pkh.to_raw_bytes().to_vec(),
        Owner::NFT(_, _) => vec![],
        Owner::Receipt(_) => vec![],
    };

    match parsed.status {
        Status::Locked => ProjectedNftData {
            address: owner_address,
            plutus_data: datum,
            operation: ProjectedNftOperation::Lock,
            ..Default::default()
        },
        Status::Unlocking {
            out_ref,
            for_how_long,
        } => ProjectedNftData {
            previous_utxo_tx_hash: out_ref.tx_id.to_raw_bytes().to_vec(),
            previous_utxo_tx_output_index: Some(out_ref.index as i64),
            address: owner_address,
            plutus_data: datum,
            operation: ProjectedNftOperation::Unlocking,
            for_how_long: Some(for_how_long as i64),
        },
    }
}

fn get_projected_nft_redeemers(redeemers: &[Redeemer]) -> Result<BTreeMap<i64, Redeem>, DbErr> {
    let mut result = BTreeMap::new();

    for redeemer in redeemers {
        if redeemer.tag != RedeemerTag::Spend {
            continue;
        }

        let plutus_data = redeemer.data.encode_fragment().unwrap();
        let plutus_data = cml_chain::plutus::PlutusData::from_bytes(plutus_data)
            .map_err(|err| DbErr::Custom(format!("Can't parse plutus data: {err}")))?;

        match Redeem::try_from(plutus_data) {
            Ok(redeem) => {
                result.insert(redeemer.index as i64, redeem);
            }
            Err(err) => {
                tracing::info!("Can't parse redeemer: {err}");
            }
        }
    }

    Ok(result)
}
