use std::collections::BTreeSet;

use entity::{
    prelude::*,
    sea_orm::{entity::*, prelude::*, ColumnTrait, Condition, DatabaseTransaction, Set},
};
use std::collections::BTreeMap;

static ADDRESS_TRUNCATE: usize = 500; // 1000 in hex

pub fn get_truncated_address(addr_bytes: &[u8]) -> &[u8] {
    &addr_bytes[0..(std::cmp::min(addr_bytes.len(), ADDRESS_TRUNCATE))]
}

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
    // theoretically, we could truncate at 2704, but we truncate at ADDRESS_TRUNCATE
    // reasons:
    // 1) Postgres has shrunk the limit in the past, so they may do it again
    // 2) Use of the INCLUDE in creating an index can increase its size
    //    So best to leave some extra room incase this is useful someday
    // 3) It's not great to hard-code a postgresql-specific limitation
    // 4) ADDRESS_TRUNCATE seems more obviously human than 2704 so maybe easier if somebody sees it
    // 5) Storing up to 2704 bytes is a waste of space since they aren't used for anything
    let truncated_addrs: Vec<&[u8]> = addresses
        .iter()
        .map(|addr| get_truncated_address(addr.as_slice()))
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

pub async fn get_outputs_for_inputs(
    inputs: &[(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )],
    txn: &DatabaseTransaction,
) -> Result<Vec<(TransactionOutputModel, TransactionModel)>, DbErr> {
    // avoid querying the DB if there were no inputs
    let has_input = inputs.iter().any(|input| !input.0.is_empty());
    if !has_input {
        return Ok(vec![]);
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

    let mut tx_outputs = TransactionOutput::find()
        .inner_join(Transaction)
        .filter(output_conditions)
        .select_with(Transaction)
        // TODO: we only actually need these columns, but sea-orm returns the full join
        // .column(TransactionOutputColumn::Id)
        // .column(TransactionOutputColumn::OutputIndex)
        // .column(TransactionOutputColumn::Payload)
        // .column(TransactionColumn::Hash)
        // .column(TransactionColumn::Id)
        // note: we can use "all" because all utxos are unique so we know:
        // 1) there won't be duplicates in the result set
        // 2) the # results == # of outputs in the filter
        .all(txn)
        .await?;

    Ok(tx_outputs
        .drain(..)
        // <tx, tx_out> is a one-to-one mapping so it's safe to flatten this
        .map(|(output, txs)| {
            if txs.len() > 1 {
                panic!();
            }
            (output, txs[0].clone())
        })
        .collect())
}

pub fn gen_input_to_output_map<'a>(
    outputs_for_inputs: &'a Vec<(TransactionOutputModel, TransactionModel)>,
) -> BTreeMap<&'a Vec<u8>, BTreeMap<i64, i64>> {
    let mut input_to_output_map = BTreeMap::<&Vec<u8>, BTreeMap<i64, i64>>::default();
    for output in outputs_for_inputs {
        input_to_output_map
            .entry(&output.1.hash)
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

    input_to_output_map
}

pub async fn insert_inputs(
    inputs: &[(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )],
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, i64>>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    // avoid querying the DB if there were no inputs
    let has_input = inputs.iter().any(|input| !input.0.is_empty());
    if !has_input {
        return Ok(());
    }

    // 3) Add inputs themselves
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
