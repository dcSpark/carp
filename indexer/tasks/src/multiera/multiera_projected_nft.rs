use cardano_projected_nft::{Owner, Redeem, State, Status};
use cml_chain::plutus::{LegacyRedeemer, PlutusData, RedeemerTag, Redeemers};
use cml_chain::transaction::DatumOption;
use cml_core::serialization::{FromBytes, Serialize};
use cml_crypto::{Ed25519KeyHash, RawBytesEncoding, TransactionHash};
use cml_multi_era::utils::{MultiEraTransactionInput, MultiEraTransactionOutput};
use cml_multi_era::MultiEraTransactionBody;
use sea_orm::{FromQueryResult, JoinType, QuerySelect, QueryTrait};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::format;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::dsl::task_macro::*;
use crate::types::AddressCredentialRelationValue;
use entity::sea_orm::Condition;
use entity::transaction_output::Model;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use tokio::sync::OnceCell;

use super::multiera_stake_credentials::MultieraStakeCredentialTask;
use super::utils::common::output_from_bytes;

use crate::config::ScriptHashConfig::ScriptHashConfig;
use crate::multiera::dex::common::filter_outputs_and_datums_by_address;
use crate::multiera::multiera_txs::MultieraTransactionTask;
use crate::multiera::multiera_used_inputs::MultieraUsedInputTask;
use crate::multiera::multiera_used_outputs::MultieraOutputTask;

carp_task! {
  name MultiEraProjectedNftTask;
  configuration ScriptHashConfig;
  doc "Parses projected NFT contract data";
  era multiera;
  dependencies [MultieraUsedInputTask, MultieraOutputTask];
  read [multiera_txs, multiera_outputs, multiera_used_inputs_to_outputs_map];
  write [];
  should_add_task |block, _properties| {
    // should trigger if any input OR output contains a projected NFT contract
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_projected_nft(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_outputs,
      &previous_data.multiera_used_inputs_to_outputs_map,
      task.config.script_hash.clone(),
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
            _ => Err("can't parse projected nft operation".to_string()),
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
    pub policy_id: String,
    pub asset_name: String,
    pub amount: i64,
    pub plutus_datum: Vec<u8>,
}

impl ProjectedNftInputsQueryOutputResult {
    pub fn subject(&self) -> String {
        format!("{}.{}", self.policy_id, self.asset_name)
    }
}

type TxInputToProjectedNft =
    BTreeMap<Vec<u8>, BTreeMap<i64, Vec<ProjectedNftInputsQueryOutputResult>>>;

async fn handle_projected_nft(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_outputs: &[TransactionOutputModel],
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
    script_hash_hex: String,
) -> Result<(), DbErr> {
    let script_hash = decode_script_hash(script_hash_hex)?;
    let projected_nft_contract_payment_cred = cml_chain::certs::Credential::new_script(script_hash);

    let mut queued_projected_nft_records = vec![];

    // spent projected nfts in current transaction
    // this requires a DB call, so we make this lazy to only compute it if really required
    let async_get_projected_nft_inputs =
        || async { get_projected_nft_inputs(db_tx, multiera_used_inputs_to_outputs_map).await };

    let once_used_projected_nfts: OnceCell<Result<TxInputToProjectedNft, DbErr>> =
        OnceCell::const_new();

    for ((tx_body, tx_witness), cardano_transaction) in block
        .1
        .transaction_bodies()
        .iter()
        .zip(block.1.transaction_witness_sets())
        .zip(multiera_txs)
    {
        if !cardano_transaction.is_valid {
            continue;
        }
        let mut partial_withdrawals_inputs: TxInputToProjectedNft = BTreeMap::new();
        // 1) Check for projected NFT inputs
        let inputs = tx_body.inputs();
        let mut parsed_inputs: Vec<(&MultiEraTransactionInput, MultiEraTransactionOutput)> = inputs
            .iter()
            .map(|input| {
                let output = &multiera_used_inputs_to_outputs_map
                    [&input.hash().unwrap().to_raw_bytes().to_vec()]
                    [&(input.index().unwrap() as i64)];
                (input, output_from_bytes(output).unwrap())
            })
            .collect::<Vec<_>>();

        // end early if we don't have projected inputs to check
        if parsed_inputs.iter().any(|(_, o)| match o.address().payment_cred() {
            Some(cred) => {
                matches!(cred, cml_chain::certs::Credential::Script { hash, .. } if *hash == script_hash)
            }
            _ => false,
        }) {
            // note: sort inputs because "spend"-type redeemers are sorted like this as well
            parsed_inputs.sort_by(
                |(left, _), (right, _)| match left.hash().cmp(&right.hash()) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => left.index().cmp(&right.index()),
                    Ordering::Greater => Ordering::Greater,
                },
            );
            let spend_redeemers = tx_witness
                .redeemers
                .as_ref()
                .map(|r| get_spend_redeemers(r))
                .map(|vec| {
                    vec.iter()
                        // we only care about redeemers that are for the projected NFT contract
                        .filter(|(_, index, _)| match &parsed_inputs[*index as usize].1.address().payment_cred() {
                            Some(cred) => {
                                matches!(cred, cml_chain::certs::Credential::Script { hash, .. } if *hash == script_hash)
                            }
                            _ => false,
                        })
                        // parse the PlutusData into the app-specific data type ("Redeem")
                        .map(|(_, index, data)| {
                            (
                                *index,
                                Redeem::try_from(data.to_cbor_bytes().as_slice())
                                    .expect("Can't parse redeemer"),
                            )
                        })
                        .collect::<BTreeMap<u64, Redeem>>()
                })
                .unwrap_or_else(BTreeMap::new);

            let used_projected_nfts = once_used_projected_nfts
                .get_or_init(async_get_projected_nft_inputs)
                .await
                .as_ref()
                .map_err(Clone::clone)?;
            // partial withdrawals inputs -- inputs which have partial withdraw = true in the redeemer
            // this function also adds claims to the events list
            partial_withdrawals_inputs = handle_claims_and_partial_withdraws(
                &parsed_inputs
                    .iter()
                    .map(|(input, _)| *input)
                    .collect::<Vec<_>>(),
                cardano_transaction,
                &spend_redeemers,
                used_projected_nfts,
                &mut queued_projected_nft_records,
            );
        }

        // 2) Check for projected NFT outputs (including additional how they may indicate partial withdrawals)
        {
            // outputs with asset data and stuff
            let outputs_map =
                get_output_index_to_outputs_map(cardano_transaction, multiera_outputs);

            // outputs that are related to projected nfts: might be locks / unlocks
            let mut projected_nft_outputs = Vec::<ProjectedNftData>::new();

            for (output_index, output) in tx_body.outputs().iter().enumerate() {
                let output_address = output.address();

                let output_payment_cred = match output_address.payment_cred() {
                    None => continue,
                    Some(pk) => pk.clone(),
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
                    extract_operation_and_datum(output, output_model, &partial_withdrawals_inputs)?;

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
                for asset in output_data.non_ada_assets.into_iter() {
                    queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                        owner_address: Set(output_data.address.clone()),
                        previous_utxo_tx_output_index: Set(
                            output_data.previous_utxo_tx_output_index
                        ),
                        previous_utxo_tx_hash: Set(output_data.previous_utxo_tx_hash.clone()),
                        hololocker_utxo_id: Set(Some(output_data.hololocker_utxo_id)),
                        tx_id: Set(cardano_transaction.id),
                        policy_id: Set(asset.policy_id),
                        asset_name: Set(asset.asset_name),
                        amount: Set(asset.amount),
                        operation: Set(output_data.operation.into()),
                        plutus_datum: Set(output_data.plutus_data.clone()),
                        for_how_long: Set(output_data.for_how_long),
                        ..Default::default()
                    });
                }
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
    partial_withdrawals_inputs: &mut TxInputToProjectedNft,
) -> Result<(), DbErr> {
    for output_data in projected_nft_outputs.iter_mut() {
        if output_data.partial_withdrawn_from_input.is_some() {
            continue;
        }

        let mut nft_data_assets = output_data.non_ada_assets.clone();
        nft_data_assets.sort_by_key(|asset| asset.subject());

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
                    .map(|w| AssetData {
                        policy_id: w.policy_id.clone(),
                        asset_name: w.asset_name.clone(),
                        amount: w.amount,
                    })
                    .collect::<Vec<_>>();
                withdrawal_assets.sort_by_key(|asset| asset.subject());

                if withdrawal_assets == nft_data_assets {
                    withdrawal_input_to_remove = Some((input_hash.clone(), *input_index));
                    output_data.previous_utxo_tx_hash.clone_from(input_hash);
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
    partial_withdrawals_inputs: &mut TxInputToProjectedNft,
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
        input_asset_to_value.insert(entry.subject(), entry.clone());
    }

    // subtract all the assets
    for output_asset_data in output_projected_nft_data.non_ada_assets.iter() {
        let output_asset_subject = output_asset_data.subject();
        input_asset_to_value
            .get_mut(&output_asset_subject)
            .ok_or(DbErr::Custom(format!(
                "Expected to see asset {output_asset_subject} in projected nft {}@{withdrawn_from_input_index}",
                hex::encode(withdrawn_from_input_hash.clone())
            )))?
            .amount -= output_asset_data.amount;
    }

    *partial_withdrawal_input = input_asset_to_value
        .values()
        .filter(|nft| nft.amount > 0)
        .cloned()
        .collect::<Vec<ProjectedNftInputsQueryOutputResult>>();

    Ok(())
}

fn decode_script_hash(script_hash: String) -> Result<cml_chain::crypto::ScriptHash, DbErr> {
    let config_script_hash = hex::decode(script_hash).map_err(|err| {
        DbErr::Custom(format!(
            "can't decode projected nft config script hash hex: {:?}",
            err
        ))
    })?;

    cml_chain::crypto::ScriptHash::from_raw_bytes(&config_script_hash)
        .map_err(|err| DbErr::Custom(format!("cml can't parse config address: {:?}", err)))
}

async fn get_projected_nft_inputs(
    db_tx: &DatabaseTransaction,
    multiera_used_inputs_to_outputs_map: &BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>>,
) -> Result<TxInputToProjectedNft, DbErr> {
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
        .column(ProjectedNftColumn::PolicyId)
        .column(ProjectedNftColumn::AssetName)
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

    let mut result: TxInputToProjectedNft = BTreeMap::new();
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
    sorted_inputs: &[&cml_multi_era::utils::MultiEraTransactionInput],
    cardano_transaction: &TransactionModel,
    redeemers: &BTreeMap<u64, Redeem>,
    used_projected_nfts: &TxInputToProjectedNft,
    queued_projected_nft_records: &mut Vec<ProjectedNftActiveModel>,
) -> TxInputToProjectedNft {
    let mut partially_withdrawn = BTreeMap::new();

    for (sorted_index, input) in sorted_inputs.iter().enumerate() {
        let input_hash = match input.hash() {
            None => continue,
            Some(hash) => hash.to_raw_bytes().to_vec(),
        };
        let input_index = match input.index() {
            None => continue,
            Some(index) => index,
        };
        let entry = if let Some(entry) = used_projected_nfts.get(&input_hash) {
            entry
        } else {
            continue;
        };

        let projected_nfts = if let Some(projected_nfts) = entry.get(&(input_index as i64)) {
            projected_nfts
        } else {
            continue;
        };

        let mut current_input_partial_withdrawal = Vec::new();

        for projected_nft in projected_nfts {
            if projected_nft.operation == i32::from(ProjectedNftOperation::Unlocking) {
                queued_projected_nft_records.push(entity::projected_nft::ActiveModel {
                    hololocker_utxo_id: Set(None),
                    tx_id: Set(cardano_transaction.id),
                    policy_id: Set(projected_nft.policy_id.clone()),
                    asset_name: Set(projected_nft.asset_name.clone()),
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
            if projected_nft.operation == i32::from(ProjectedNftOperation::Lock)
                && redeemers
                    .get(&(sorted_index as u64))
                    .unwrap()
                    .partial_withdraw
            {
                current_input_partial_withdrawal.push(projected_nft.clone());
            }
        }

        if !current_input_partial_withdrawal.is_empty() {
            *partially_withdrawn
                .entry(input_hash)
                .or_insert(BTreeMap::new())
                .entry(input_index as i64)
                .or_default() = current_input_partial_withdrawal;
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

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AssetData {
    pub policy_id: String,
    pub asset_name: String,
    pub amount: i64,
}

impl AssetData {
    pub fn subject(&self) -> String {
        format!("{}.{}", self.policy_id, self.asset_name)
    }

    pub fn from_subject(subject: String, amount: i64) -> Result<AssetData, DbErr> {
        let mut split = subject.split('.');
        let policy_id = if let Some(policy_id_hex) = split.next() {
            policy_id_hex.to_string()
        } else {
            return Err(DbErr::Custom(
                "No policy id found in asset subject".to_string(),
            ));
        };
        let asset_name = if let Some(asset_name) = split.next() {
            asset_name.to_string()
        } else {
            return Err(DbErr::Custom(
                "No asset name found in asset subject".to_string(),
            ));
        };
        if let Some(next) = split.next() {
            return Err(DbErr::Custom(format!(
                "Extra information is found in asset: {next}"
            )));
        }
        Ok(AssetData {
            policy_id,
            asset_name,
            amount,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct ProjectedNftData {
    pub previous_utxo_tx_hash: Vec<u8>, // warning: this isn't nullable, but can be an empty vector to represent null
    pub previous_utxo_tx_output_index: Option<i64>,
    pub address: Vec<u8>,
    pub plutus_data: Vec<u8>,
    pub operation: ProjectedNftOperation,
    pub for_how_long: Option<i64>,
    // this field is set only on unlocking outputs that were created through partial withdraw
    pub partial_withdrawn_from_input: Option<(Vec<u8>, i64)>,
    pub non_ada_assets: Vec<AssetData>,
    pub hololocker_utxo_id: i64,
}

fn extract_operation_and_datum(
    output: &MultiEraTransactionOutput,
    output_model: entity::transaction_output::Model,
    partial_withdrawals: &TxInputToProjectedNft,
) -> Result<ProjectedNftData, DbErr> {
    let output = match output {
        MultiEraTransactionOutput::Byron(_) => {
            return Err(DbErr::Custom(
                "got byron block in projected nft task".to_string(),
            ));
        }
        MultiEraTransactionOutput::Shelley(shelley) => shelley.clone(),
    };
    let datum_option = match output.datum() {
        Some(datum) => datum,
        None => {
            return Ok(ProjectedNftData {
                operation: ProjectedNftOperation::NoDatum,
                ..Default::default()
            });
        }
    };

    let datum = match datum_option {
        DatumOption::Hash { datum_hash, .. } => {
            return Ok(ProjectedNftData {
                plutus_data: datum_hash.to_raw_bytes().to_vec(),
                // the contract expects inline datums only
                operation: ProjectedNftOperation::NotInlineDatum,
                ..Default::default()
            });
        }
        DatumOption::Datum { datum, .. } => datum,
    };

    let datum_bytes = datum.to_cbor_bytes();

    let parsed = match cardano_projected_nft::State::try_from(datum_bytes.as_slice()) {
        Ok(parsed) => parsed,
        Err(_) => {
            return Ok(ProjectedNftData {
                plutus_data: datum_bytes,
                operation: ProjectedNftOperation::ParseError,
                ..Default::default()
            });
        }
    };

    let owner_address = match parsed.owner {
        Owner::PKH(pkh) => Ed25519KeyHash::from_hex(&pkh.to_hex())
            .map_err(|_err| DbErr::Custom("can't parse pkh".to_string()))?
            .to_raw_bytes()
            .to_vec(),
        Owner::NFT(_, _) => vec![],
        Owner::Receipt(_) => vec![],
    };

    let non_ada_assets = output
        .amount()
        .multiasset
        .iter()
        .flat_map(|(policy_id, assets)| {
            assets.iter().map(|(asset_name, value)| AssetData {
                policy_id: policy_id.to_hex(),
                asset_name: hex::encode(asset_name.to_raw_bytes()),
                amount: *value as i64,
            })
        })
        .collect::<Vec<AssetData>>();
    let result = match parsed.status {
        Status::Locked => ProjectedNftData {
            address: owner_address,
            plutus_data: datum_bytes,
            operation: ProjectedNftOperation::Lock,
            hololocker_utxo_id: output_model.id,
            non_ada_assets,
            ..Default::default()
        },
        Status::Unlocking {
            out_ref,
            for_how_long,
        } => {
            let out_ref_tx_id = TransactionHash::from_hex(&out_ref.tx_id.to_hex())
                .map_err(|_err| DbErr::Custom("can't parse tx hash".to_string()))?
                .to_raw_bytes()
                .to_vec();
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
                plutus_data: datum_bytes,
                operation: ProjectedNftOperation::Unlocking,
                for_how_long: Some(match for_how_long.as_u64() {
                    None => {
                        return Err(DbErr::Custom(format!(
                            "for how long does not fit into u64 {}",
                            for_how_long
                        )))
                    }
                    Some(ok) => ok as i64,
                }),
                hololocker_utxo_id: output_model.id,
                partial_withdrawn_from_input: partial_withdrawn_from,
                non_ada_assets,
            }
        }
    };

    Ok(result)
}

fn get_spend_redeemers(redeemers: &Redeemers) -> Vec<(RedeemerTag, u64, &PlutusData)> {
    match redeemers {
        Redeemers::ArrLegacyRedeemer {
            arr_legacy_redeemer,
            arr_legacy_redeemer_encoding: _,
        } => arr_legacy_redeemer
            .iter()
            .map(|redeemer| (redeemer.tag, redeemer.index, &redeemer.data))
            .filter(|(tag, _, _)| *tag == RedeemerTag::Spend)
            .collect(),
        Redeemers::MapRedeemerKeyToRedeemerVal {
            map_redeemer_key_to_redeemer_val,
            map_redeemer_key_to_redeemer_val_encoding: _,
        } => map_redeemer_key_to_redeemer_val
            .iter()
            .map(|(key, val)| (key.tag, key.index, &val.data))
            .filter(|(tag, _, _)| *tag == RedeemerTag::Spend)
            .collect(),
    }
}
