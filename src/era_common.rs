use entity::{
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, JoinType, QuerySelect, Set},
};
use futures::future::join_all;
use pallas::crypto::hash::Hash;

use crate::{relation_map::RelationMap, types::TxCredentialRelationValue};

pub async fn insert_address(
    payload: &mut Vec<u8>,
    txn: &DatabaseTransaction,
) -> Result<AddressModel, DbErr> {
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
    payload.truncate(500);

    // note: in the usual case, the address will already be in the DB when we query it
    // that means it's faster to use find instead of write(on conflict do nothing)
    // since "do nothing" returns None, a conflict mean we would have to use find as a fallback
    // meaning the "on conflict do nothing" requires 2 queries in the usual case instead of 1
    let addr = Address::find()
        .filter(AddressColumn::Payload.eq(payload.clone()))
        // note: okay to use "all" since we're querying a unique key
        // and "all" is faster than "first" if you know it will return a single result
        .all(txn)
        .await?;

    if let Some(addr) = addr.first() {
        Ok(addr.clone())
    } else {
        let address = AddressActiveModel {
            payload: Set(payload.clone()),
            ..Default::default()
        };

        let address = address.insert(txn).await?;
        Ok(address)
    }
}

pub async fn insert_inputs(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    inputs: &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    // 1) Get the UTXO this input is spending
    let tx_outputs = join_all(inputs.iter().map(|input| {
        TransactionOutput::find()
            .inner_join(Transaction)
            .filter(TransactionOutputColumn::OutputIndex.eq(input.index))
            .filter(TransactionColumn::Hash.eq(input.transaction_id.to_vec()))
            .one(txn)
    }))
    .await;

    // 2) Associate any relation for credentials
    let related_credentials = join_all(
        tx_outputs
            .iter()
            // Byron addresses don't contain stake credentials, so we can skip them
            .filter(|&tx_output| {
                let model = tx_output.as_ref().unwrap().as_ref().unwrap();
                let is_byron = match cardano_multiplatform_lib::TransactionOutput::from_bytes(
                    model.payload.clone(),
                ) {
                    Ok(parsed_output) => parsed_output.address().as_byron().is_some(),
                    // TODO: remove this once we've parsed the genesis block correctly instead of inserting dummy data
                    Err(_) => true,
                };
                !is_byron
            })
            .map(|tx_output| {
                let model = tx_output.as_ref().unwrap().as_ref().unwrap();

                // 2) Get the stake credential for the UTXO being spent
                StakeCredential::find()
                    .inner_join(AddressCredential)
                    .join(
                        JoinType::InnerJoin,
                        AddressCredentialRelation::Address.def(),
                    )
                    .join(
                        JoinType::InnerJoin,
                        AddressRelation::TransactionOutput.def(),
                    )
                    .filter(TransactionOutputColumn::Id.eq(model.id))
                    .all(txn)
            }),
    )
    .await;

    // 3) Associate the stake credentials to this transaction
    for stake_credentials in related_credentials {
        for stake_credential in stake_credentials.unwrap() {
            vkey_relation_map.add_relation(
                stake_credential.id,
                &stake_credential.credential,
                TxCredentialRelationValue::Input,
            );
        }
    }

    // 4) Add inputs themselves
    TransactionInput::insert_many(
        inputs
            .iter()
            .zip(tx_outputs)
            .enumerate()
            .map(|(idx, pair)| TransactionInputActiveModel {
                utxo_id: Set(pair.1.as_ref().unwrap().as_ref().unwrap().id),
                tx_id: Set(tx_id),
                input_index: Set(idx as i32),
                ..Default::default()
            }),
    )
    .exec(txn)
    .await?;

    Ok(())
}
