use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use entity::{
    prelude::*,
    sea_orm::{
        entity::*, prelude::*, ColumnTrait, Condition, DatabaseTransaction, JoinType, QuerySelect,
        Set,
    },
};
use std::collections::BTreeMap;

use crate::{relation_map::RelationMap, types::TxCredentialRelationValue};

pub async fn insert_addresses(
    addresses: &BTreeSet<Vec<u8>>,
    txn: &DatabaseTransaction,
) -> Result<(Vec<AddressModel>, BTreeMap<Vec<u8>, AddressModel>), DbErr> {
    if addresses.is_empty() {
        return Ok((vec![], BTreeMap::default()));
    }
    // During the Byron era of Cardano,
    // Addresses had a feature where you could add extra metadata in them
    // The amount of metadata you could insert was not capped
    // So some addresses got generated which are really large
    // However, Postgres btree v4 has a maximum size of 2704 for an index
    // Since these addresses can't be spent anyway, we just truncate them
    // theoretically, we could truncate at 2704, but we truncate at 500
    // reasons:
    // 1) Postgres has shrunk the limit in the past, so they may do it again
    // 2) Use of the INCLUDE in creating an index can increase its size
    //    So best to leave some extra room incase this is useful someday
    // 3) It's not great to hard-code a postgresql-specific limitation
    // 4) 500 seems more obviously human than 2704 so maybe easier if somebody sees it
    // 5) Storing up to 2704 bytes is a waste of space since they aren't used for anything
    let truncated_addrs: Vec<&[u8]> = addresses
        .iter()
        .map(|addr_bytes| &addr_bytes[0..(std::cmp::min(addr_bytes.len(), 500))])
        .collect();

    // deduplicate addresses to avoid re-querying the same address many times
    // useful not only as a perf improvement, but also avoids parallel queries writing to the same row
    let deduplicated = BTreeSet::<_>::from_iter(truncated_addrs.clone());

    let mut result_map = BTreeMap::<Vec<u8>, AddressModel>::default();

    // 1) Add addresses that were already in the DB
    {
        // note: in the usual case, the address will already be in the DB when we query it
        // that means it's faster to use find instead of write(on conflict do nothing)
        // since "do nothing" returns None, a conflict mean we would have to use find as a fallback
        // meaning the "on conflict do nothing" requires 2 queries in the usual case instead of 1

        // note: okay to batch use "all" since we're querying unique keys
        let mut found_addresses = Address::find()
            .filter(Condition::any().add(AddressColumn::Payload.is_in(deduplicated.clone())))
            .all(txn)
            .await?;

        // add addresses that already existed previously directly to the result
        result_map.extend(
            found_addresses
                .drain(..)
                .map(|model| (model.payload.clone(), model)),
        );
    }

    // 2) Add addresses that weren't in the DB
    let mut additions: Vec<AddressModel> = vec![];
    {
        // check which addresses weren't found in the DB and prepare to add them
        let addrs_to_add: Vec<AddressActiveModel> = deduplicated
            .iter()
            .filter(|&&addr| !result_map.contains_key(addr))
            .map(|addr| AddressActiveModel {
                payload: Set(addr.to_vec()),
                ..Default::default()
            })
            .collect();

        // add the new entires into the DB, then add them to our result mapping
        if !addrs_to_add.is_empty() {
            additions.extend(
                Address::insert_many(addrs_to_add)
                    .exec_many_with_returning(txn)
                    .await?,
            );
            additions.iter().for_each(|model| {
                result_map.insert(model.payload.clone(), model.clone());
            });
        }
    }

    Ok((additions, result_map))
}

pub async fn insert_inputs(
    vkey_relation_map: &mut RelationMap,
    inputs: &Vec<(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    // avoid querying the DB if there were no inputs
    let has_input = inputs.iter().any(|input| input.0.len() > 0);
    if !has_input {
        return Ok(());
    }

    // 1) Get the UTXO this input is spending
    let mut output_conditions = Condition::any();

    // note: we don't need to deduplicate the conditions because every UTXO can only be spent once
    // so we know all these pairs are disjoint amongst all transactions
    for input in inputs.iter().flat_map(|inputs| inputs.0.iter()) {
        output_conditions = output_conditions.add(
            Condition::all()
                .add(TransactionOutputColumn::OutputIndex.eq(input.index))
                .add(TransactionColumn::Hash.eq(input.transaction_id.to_vec())),
        );
    }

    let tx_outputs = TransactionOutput::find()
        .inner_join(Transaction)
        .filter(output_conditions)
        .select_with(Transaction)
        .column(TransactionOutputColumn::Id)
        .column(TransactionOutputColumn::OutputIndex)
        .column(TransactionOutputColumn::Payload)
        .column(TransactionColumn::Hash)
        .column(TransactionColumn::Id)
        // note: we can use "all" because all utxos are unique so we know:
        // 1) there won't be duplicates in the result set
        // 2) the # results == # of outputs in the filter
        .all(txn)
        .await?;

    // 2) Creating the mappings we need to map from our DB results back to the original function input

    let mut input_to_output_map = BTreeMap::<&Vec<u8>, BTreeMap<i64, i64>>::default();
    for output in &tx_outputs {
        input_to_output_map
            .entry(&output.1.first().unwrap().hash)
            .and_modify(|output_index_map| {
                // note: we can insert right away instead of doing a 2nd lookup
                // because the pair <payload, output_index> is unique
                output_index_map.insert(output.0.output_index as i64, output.0.id);
            })
            .or_insert({
                let mut output_index_map = BTreeMap::<i64, i64>::default();
                output_index_map.insert(output.0.output_index as i64, output.0.id);
                output_index_map
            });
    }

    let mut output_to_input_tx = BTreeMap::<i64, i64>::default();
    for input_tx_pair in inputs.iter() {
        for input in input_tx_pair.0.iter() {
            let output_id =
                input_to_output_map[&input.transaction_id.to_vec()][&(input.index as i64)];
            output_to_input_tx.insert(output_id, input_tx_pair.1);
        }
    }

    // 3) Get the credential for any Shelley-era address

    let shelley_output_ids: Vec<i64> = tx_outputs
        .iter()
        // Byron addresses don't contain stake credentials, so we can skip them
        .filter(|&tx_output| {
            let is_byron = match cardano_multiplatform_lib::TransactionOutput::from_bytes(
                tx_output.0.payload.clone(),
            ) {
                Ok(parsed_output) => parsed_output.address().as_byron().is_some(),
                // TODO: remove this once we've parsed the genesis block correctly instead of inserting dummy data
                Err(_) => true,
            };
            !is_byron
        })
        .map(|output| output.0.id)
        .collect();

    if shelley_output_ids.len() > 0 {
        let related_credentials = StakeCredential::find()
            .inner_join(AddressCredential)
            .join(
                JoinType::InnerJoin,
                AddressCredentialRelation::Address.def(),
            )
            .join(
                JoinType::InnerJoin,
                AddressRelation::TransactionOutput.def(),
            )
            .filter(
                Condition::any().add(TransactionOutputColumn::Id.is_in(shelley_output_ids.clone())),
            )
            // we need to know which OutputId every credential is for so we can know which tx these creds are related to
            .select_with(TransactionOutput)
            .column(StakeCredentialColumn::Id)
            .column(StakeCredentialColumn::Credential)
            .column(TransactionOutputColumn::Id)
            .all(txn)
            .await?;

        // 4) Associate the stake credentials to this transaction
        if related_credentials.len() > 0 {
            for stake_credentials in &related_credentials {
                // recall: the same stake credential could have shown up in multiple outputs
                for output in stake_credentials.1.iter() {
                    vkey_relation_map.add_relation(
                        output_to_input_tx[&output.id],
                        &stake_credentials.0.credential,
                        TxCredentialRelationValue::Input,
                    );
                }
            }
        }
    }

    // 5) Add inputs themselves
    TransactionInput::insert_many(
        inputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(|((idx, input), tx_id)| TransactionInputActiveModel {
                utxo_id: Set(
                    input_to_output_map[&input.transaction_id.to_vec()][&(input.index as i64)]
                ),
                tx_id: Set(tx_id),
                input_index: Set(idx as i32),
                ..Default::default()
            }),
    )
    .exec(txn)
    .await?;

    Ok(())
}
