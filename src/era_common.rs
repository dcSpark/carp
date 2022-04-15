use entity::{
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, JoinType, QuerySelect, Set},
};
use pallas::crypto::hash::Hash;

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

pub async fn insert_input(
    tx: &TransactionModel,
    index_in_input: i32,
    index_in_output: u64,
    tx_hash: &Hash<32>,
    txn: &DatabaseTransaction,
) -> Result<Vec<entity::stake_credential::Model>, DbErr> {
    let mut result: Vec<entity::stake_credential::Model> = vec![];

    // 1) Get the UTXO this input is spending
    let tx_output = TransactionOutput::find()
        .inner_join(Transaction)
        .filter(TransactionOutputColumn::OutputIndex.eq(index_in_output))
        .filter(TransactionColumn::Hash.eq(tx_hash.to_vec()))
        // note: we know this input exists and "all" is faster than "one" if we know the result exists
        .all(txn)
        .await?;

    let tx_output = tx_output.first().cloned().unwrap();

    let is_byron = match cardano_multiplatform_lib::TransactionOutput::from_bytes(tx_output.payload)
    {
        Ok(parsed_output) => parsed_output.address().as_byron().is_some(),
        // TODO: remove this once we've parsed the genesis block correctly instead of inserting dummy data
        Err(_) => true,
    };
    // Byron addresses don't contain stake credentials, so we can skip them
    if !is_byron {
        // 2) Get the stake credential for the UTXO being spent
        let stake_credentials = StakeCredential::find()
            .inner_join(AddressCredential)
            .join(
                JoinType::InnerJoin,
                AddressCredentialRelation::Address.def(),
            )
            .join(
                JoinType::InnerJoin,
                AddressRelation::TransactionOutput.def(),
            )
            .filter(TransactionOutputColumn::Id.eq(tx_output.id))
            .all(txn)
            .await?;

        // 3) Associate the stake credentials to this transaction
        for stake_credential in stake_credentials {
            result.push(stake_credential);
        }
    }

    // 4) Add input itself
    let tx_input = TransactionInputActiveModel {
        utxo_id: Set(tx_output.id),
        tx_id: Set(tx.id),
        input_index: Set(index_in_input),
        ..Default::default()
    };

    tx_input.save(txn).await?;

    Ok(result)
}
