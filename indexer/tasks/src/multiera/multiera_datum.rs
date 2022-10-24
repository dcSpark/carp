use std::collections::BTreeSet;

use super::multiera_txs::MultieraTransactionTask;
use super::{
    multiera_used_inputs::add_input_relations, multiera_used_outputs::MultieraOutputTask,
    relation_map::RelationMap,
};
use crate::config::ReadonlyConfig::ReadonlyConfig;
use entity::sea_orm::QuerySelect;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, JoinType, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::ComputeHash;
use pallas::ledger::{
    primitives::babbage::{DatumHash, DatumOption},
    traverse::{MultiEraBlock, OutputRef},
};

use crate::dsl::task_macro::*;

carp_task! {
name MultieraDatumTask;
configuration ReadonlyConfig;
doc "Adds datum and datum hashes";
era multiera;
dependencies [MultieraTransactionTask];
read [multiera_txs];
write [];
should_add_task |block, _properties| {
  block.1.txs().iter().any(|tx| {
    !tx.plutus_data().is_empty() || tx.outputs().iter().any(|output| output.datum().is_some())
  })
};
execute |previous_data, task| handle_datum(
    task.db_tx,
    task.block,
    &previous_data.multiera_txs,
    task.config.readonly
);
merge_result |previous_data, _result| {
};
}

async fn handle_datum(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
    readonly: bool,
) -> Result<(), DbErr> {
    let mut hash_to_tx = BTreeMap::<DatumHash, i64>::new();
    // recall: tx may contain datum hash only w/ datum only appearing in a later tx
    let mut hash_to_data = BTreeMap::<DatumHash, Vec<u8>>::new();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        for datum in tx_body.plutus_data() {
            let hash = datum.compute_hash();
            hash_to_tx
                .entry(hash)
                .or_insert_with(|| cardano_transaction.id);
            hash_to_data
                .entry(hash)
                .or_insert_with(|| datum.encode_fragment().unwrap());
        }
        for output in tx_body.outputs().iter() {
            match output.datum().as_ref() {
                Some(DatumOption::Hash(hash)) => {
                    hash_to_tx
                        .entry(*hash)
                        .or_insert_with(|| cardano_transaction.id);
                }
                Some(DatumOption::Data(datum)) => {
                    let hash = datum.compute_hash();
                    hash_to_tx
                        .entry(hash)
                        .or_insert_with(|| cardano_transaction.id);
                    hash_to_data
                        .entry(hash)
                        .or_insert_with(|| datum.0.encode_fragment().unwrap());
                }
                None => {}
            };
        }
    }

    if hash_to_tx.is_empty() {
        return Ok(());
    }

    let mut hash_to_id = BTreeMap::<Vec<u8>, i64>::new();
    let mut existing_full_datums = BTreeSet::<i64>::new();
    // 1) Get hashes that were already in the DB
    {
        let mut found_hashes =
            PlutusDataHash::find()
                .join(JoinType::LeftJoin, PlutusDataHashRelation::PlutusData.def())
                .filter(Condition::any().add(
                    PlutusDataHashColumn::Hash.is_in(hash_to_tx.keys().map(|hash| hash.as_ref())),
                ))
                // TODO: would be more efficient to just select the ID field of PlutusData
                // to avoid having to return large datum objects that we just ignore in the SQL query
                .select_with(PlutusData)
                .all(db_tx)
                .await?;

        for (datum_hash, datums) in found_hashes.iter() {
            if !datums.is_empty() {
                existing_full_datums.insert(datum_hash.id);
            }
        }
        hash_to_id.extend(
            found_hashes
                .drain(..)
                .map(|entry| (entry.0.hash, entry.0.id)),
        );
    }
    // 2) Add hashes that were not already in the DB
    {
        let keys_to_add: Vec<&DatumHash> = hash_to_tx
            .keys()
            .filter(|key| !hash_to_id.contains_key(key.as_ref()))
            .collect();
        let to_add: Vec<PlutusDataHashActiveModel> = keys_to_add
            .iter()
            .map(|key| PlutusDataHashActiveModel {
                hash: Set(key.to_vec()),
                first_tx: Set(*hash_to_tx.get(key).unwrap()),
                ..Default::default()
            })
            .collect();

        if !to_add.is_empty() {
            if readonly {
                panic!(
                    "{} in readonly mode, but unknown Plutus datum hashes were found: {:?}",
                    "MultieraDatumTask",
                    keys_to_add.iter().map(|key| hex::encode(key.as_ref()))
                );
            }
            let mut new_entries = PlutusDataHash::insert_many(to_add)
                .exec_many_with_returning(db_tx)
                .await?;
            for entry in new_entries.drain(..) {
                hash_to_id.insert(entry.hash, entry.id);
            }
        }
    }
    // 3) Add datum
    {
        let to_add = hash_to_data.iter().fold(vec![], |mut acc, next| {
            let datum_hash_id = hash_to_id.get(next.0.as_ref()).unwrap();
            match existing_full_datums.get(datum_hash_id) {
                None => {
                    acc.push(PlutusDataActiveModel {
                        id: Set(*datum_hash_id),
                        data: Set(next.1.encode_fragment().unwrap()),
                    });
                    acc
                }
                Some(_) => acc,
            }
        });
        if !to_add.is_empty() {
            if readonly {
                panic!(
                    "{} in readonly mode, but unknown Plutus data content was found for hashes: {:?}",
                    "MultieraDatumTask",
                    to_add.iter().map(|entry| {
                      let hashes: Vec<String> = hash_to_id.iter()
                        .filter_map(|(key, &val)| if val == *entry.id.as_ref() { Some(hex::encode(key)) } else { None })
                        .collect();
                      hashes
                    })
                );
            }
            PlutusData::insert_many(to_add)
                .exec_many_with_returning(db_tx)
                .await?;
        }
    }

    Ok(())
}
