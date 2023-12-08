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
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::format;

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

use crate::config::AddressConfig::AddressConfig;
use crate::multiera::dex::common::filter_outputs_and_datums_by_address;
use crate::multiera::multiera_txs::MultieraTransactionTask;
use crate::multiera::multiera_used_inputs::MultieraUsedInputTask;
use crate::multiera::multiera_used_outputs::MultieraOutputTask;

carp_task! {
  name MultiEraProjectedNftTask;
  configuration AddressConfig;
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
    pub plutus_datum: Vec<u8>,
}

async fn handle_projected_nft(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_outputs: &[TransactionOutputModel],
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    address: String,
) -> Result<(), DbErr> {
    let projected_nft_contract_payment_cred = get_payment_cred(address)?;

    // spent projected nfts in current transaction
    let used_projected_nfts =
        get_projected_nft_inputs(db_tx, multiera_used_inputs_to_outputs_map).await?;

    let mut queued_projected_nft_records = vec![];

    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        // redeemers are needed to identify whil of the projected nfts are partial withdrawals
        let redeemers = tx_body
            .redeemers()
            .map(get_projected_nft_redeemers)
            .unwrap_or(Ok(BTreeMap::new()))?;

        // partial withdrawals inputs -- inputs which have partial withdraw = true in the redeemer
        // this function also adds claims to the events list
        let mut partial_withdrawals_inputs = handle_claims_and_partial_withdraws(
            tx_body,
            cardano_transaction,
            &redeemers,
            &used_projected_nfts,
            &mut queued_projected_nft_records,
        );

        // outputs with asset data and stuff
        let outputs_map = get_output_index_to_outputs_map(cardano_transaction, multiera_outputs);

        // outputs that are related to projected nfts: might be locks / unlocks
        let mut projected_nft_outputs = Vec::<ProjectedNftData>::new();

        for (output_index, output) in tx_body.outputs().iter().enumerate() {
            let output_address = output
                .address()
                .map_err(|err| DbErr::Custom(format!("invalid pallas address: {}", err)))?
                .to_vec();

            let output_address =
                cardano_multiplatform_lib::address::Address::from_bytes(output_address)
                    .map_err(|err| DbErr::Custom(format!("cml can't parse address: {}", err)))?;
            let output_payment_cred = match output_address.payment_cred() {
                None => continue,
                Some(pk) => pk,
            };

            if output_payment_cred != projected_nft_contract_payment_cred {
                // the output doesn't relate to projected nft contract -> we don't care about it
                continue;
            }

            // current output details
            let output_model = outputs_map
                .get(&(output_index as i32))
                .ok_or(DbErr::RecordNotFound(format!(
                    "can't find output with index {output_index} of tx {}",
                    cardano_transaction.id
                )))?
                .clone();

            // parse the state and fetch projected nft details
            let projected_nft_data =
                extract_operation_and_datum(output, output_model, &partial_withdrawals_inputs);

            // if projected nft data is unlocking output that is created via partial withdrawal
            // then we reduce associated partial withdrawal input balance by the amounts that are being unlocked
            // this way we will be able to find the corresponding lock output that will have the rest of the balance
            handle_partial_withdraw(&projected_nft_data, &mut partial_withdrawals_inputs)?;

            projected_nft_outputs.push(projected_nft_data);
        }

        find_lock_outputs_for_corresponding_partial_withdrawals(
            &mut projected_nft_outputs,
            &mut partial_withdrawals_inputs,
        )?;

        if !partial_withdrawals_inputs.is_empty() {
            return Err(DbErr::Custom(format!("Partial withdrawals must be empty at the end of projected nft processing, while contains: {}", partial_withdrawals_inputs.keys().map(hex::encode).fold(String::new(), |acc, key| format!("{acc},{key}")))));
        }

        for output_data in projected_nft_outputs.into_iter() {
            for (asset_name, asset_value) in output_data.non_ada_assets.into_iter() {
                queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                    owner_address: Set(output_data.address.clone()),
                    previous_utxo_tx_output_index: Set(output_data.previous_utxo_tx_output_index),
                    previous_utxo_tx_hash: Set(output_data.previous_utxo_tx_hash.clone()),
                    hololocker_utxo_id: Set(Some(output_data.hololocker_utxo_id)),
                    tx_id: Set(cardano_transaction.id),
                    asset: Set(asset_name),
                    amount: Set(asset_value),
                    operation: Set(output_data.operation.into()),
                    plutus_datum: Set(output_data.plutus_data.clone()),
                    for_how_long: Set(output_data.for_how_long),
                    ..Default::default()
                });
            }
        }
    }

    if !queued_projected_nft_records.is_empty() {
        ProjectedNft::insert_many(queued_projected_nft_records)
            .exec(db_tx)
            .await?;
    }

    Ok(())
}

fn find_lock_outputs_for_corresponding_partial_withdrawals(
    projected_nft_outputs: &mut [ProjectedNftData],
    partial_withdrawals_inputs: &mut BTreeMap<
        Vec<u8>,
        BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>,
    >,
) -> Result<(), DbErr> {
    for output_data in projected_nft_outputs.iter_mut() {
        if output_data.partial_withdrawn_from_input.is_some() {
            continue;
        }

        let mut nft_data_assets = output_data.non_ada_assets.clone();
        nft_data_assets.sort_by_key(|(name, _)| name.clone());

        let mut withdrawal_input_to_remove: Option<(Vec<u8>, i64)> = None;

        for (input_hash, withdrawal) in partial_withdrawals_inputs.iter() {
            for (input_index, withdrawal) in withdrawal.iter() {
                let withdrawal_data = withdrawal.first().ok_or(DbErr::Custom(format!(
                    "Expected to see an asset in utxo {}@{input_index}",
                    hex::encode(input_hash.clone())
                )))?;
                if withdrawal_data.plutus_datum != output_data.plutus_data
                    || withdrawal_data.owner_address != output_data.address
                {
                    continue;
                }

                let mut withdrawal_assets = withdrawal
                    .iter()
                    .map(|w| (w.asset.clone(), w.amount))
                    .collect::<Vec<_>>();
                withdrawal_assets.sort_by_key(|(name, _)| name.clone());

                if withdrawal_assets == nft_data_assets {
                    withdrawal_input_to_remove = Some((input_hash.clone(), *input_index));
                    output_data.previous_utxo_tx_hash = input_hash.clone();
                    output_data.previous_utxo_tx_output_index = Some(*input_index);
                    break;
                }
            }
        }

        if let Some((hash, index)) = withdrawal_input_to_remove {
            partial_withdrawals_inputs
                .get_mut(&hash)
                .unwrap()
                .remove(&index);
            if partial_withdrawals_inputs
                .get_mut(&hash)
                .unwrap()
                .is_empty()
            {
                partial_withdrawals_inputs.remove(&hash);
            }
        }
    }

    Ok(())
}

fn handle_partial_withdraw(
    output_projected_nft_data: &ProjectedNftData,
    partial_withdrawals_inputs: &mut BTreeMap<
        Vec<u8>,
        BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>,
    >,
) -> Result<(), DbErr> {
    let (withdrawn_from_input_hash, withdrawn_from_input_index) =
        if let Some((hash, index)) = &output_projected_nft_data.partial_withdrawn_from_input {
            (hash, index)
        } else {
            return Ok(());
        };

    // get associated projected nft input
    let partial_withdrawal_input = partial_withdrawals_inputs
        .get_mut(&withdrawn_from_input_hash.clone())
        .ok_or(DbErr::Custom(format!(
            "projected nft input hash {} should always exist",
            hex::encode(withdrawn_from_input_hash.clone())
        )))?
        .get_mut(withdrawn_from_input_index)
        .ok_or(DbErr::Custom(format!(
            "projected nft input with hash {} and index {} should always exist",
            hex::encode(withdrawn_from_input_hash.clone()),
            withdrawn_from_input_index
        )))?;

    // make a balance map
    let mut input_asset_to_value = HashMap::<String, ProjectedNftInputsQueryOutputResult>::new();
    for entry in partial_withdrawal_input.iter() {
        input_asset_to_value.insert(entry.asset.clone(), entry.clone());
    }

    // subtract all the assets
    for (output_asset_name, output_asset_value) in output_projected_nft_data.non_ada_assets.iter() {
        input_asset_to_value
            .get_mut(&output_asset_name.clone())
            .ok_or(DbErr::Custom(format!(
                "Expected to see asset {output_asset_name} in projected nft {}@{withdrawn_from_input_index}",
                hex::encode(withdrawn_from_input_hash.clone())
            )))?
            .amount -= output_asset_value;
    }

    *partial_withdrawal_input = input_asset_to_value
        .values()
        .filter(|nft| nft.amount > 0)
        .cloned()
        .collect::<Vec<ProjectedNftInputsQueryOutputResult>>();

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
        .column(ProjectedNftColumn::PlutusDatum)
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
) -> BTreeMap<Vec<u8>, BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>> {
    let mut partially_withdrawn = BTreeMap::new();

    let mut sorted_inputs = tx_body.inputs();
    sorted_inputs.sort_by(|left, right| match left.hash().cmp(right.hash()) {
        Ordering::Less => Ordering::Less,
        Ordering::Equal => left.index().cmp(&right.index()),
        Ordering::Greater => Ordering::Greater,
    });

    for (input_index, input) in sorted_inputs.iter().enumerate() {
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

        let mut current_input_partial_withrawal = Vec::new();

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
                    current_input_partial_withrawal.push(projected_nft.clone());
                }
            }
        }

        if !current_input_partial_withrawal.is_empty() {
            *partially_withdrawn
                .entry(input.hash().to_vec())
                .or_insert(BTreeMap::new())
                .entry(input.index() as i64)
                .or_default() = current_input_partial_withrawal;
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
    // this field is set only on unlocking outputs that were created through partial withdraw
    pub partial_withdrawn_from_input: Option<(Vec<u8>, i64)>,
    pub non_ada_assets: Vec<(String, i64)>,
    pub hololocker_utxo_id: i64,
}

fn extract_operation_and_datum(
    output: &MultiEraOutput,
    output_model: entity::transaction_output::Model,
    partial_withdrawals: &BTreeMap<
        Vec<u8>,
        BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>,
    >,
) -> ProjectedNftData {
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

    let non_ada_assets = output
        .non_ada_assets()
        .iter()
        .map(|asset| {
            (
                asset.subject(),
                match asset {
                    Asset::Ada(value) => *value as i64,
                    Asset::NativeAsset(_, _, value) => *value as i64,
                },
            )
        })
        .collect::<Vec<(String, i64)>>();
    match parsed.status {
        Status::Locked => ProjectedNftData {
            address: owner_address,
            plutus_data: datum,
            operation: ProjectedNftOperation::Lock,
            hololocker_utxo_id: output_model.id,
            non_ada_assets,
            ..Default::default()
        },
        Status::Unlocking {
            out_ref,
            for_how_long,
        } => {
            let out_ref_tx_id = out_ref.tx_id.to_raw_bytes().to_vec();
            let partial_withdrawn_from =
                partial_withdrawals.get(&out_ref_tx_id).and_then(|inner| {
                    if inner.contains_key(&(out_ref.index as i64)) {
                        Some((out_ref_tx_id.clone(), out_ref.index as i64))
                    } else {
                        None
                    }
                });

            ProjectedNftData {
                previous_utxo_tx_hash: out_ref_tx_id,
                previous_utxo_tx_output_index: Some(out_ref.index as i64),
                address: owner_address,
                plutus_data: datum,
                operation: ProjectedNftOperation::Unlocking,
                for_how_long: Some(for_how_long as i64),
                hololocker_utxo_id: output_model.id,
                partial_withdrawn_from_input: partial_withdrawn_from,
                non_ada_assets,
            }
        }
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
