use std::collections::BTreeSet;

use super::{
    multiera_used_inputs::add_input_relations, multiera_used_outputs::MultieraOutputTask,
    relation_map::RelationMap,
};
use crate::config::EmptyConfig::EmptyConfig;
use entity::sea_orm::QuerySelect;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, Condition, DatabaseTransaction, JoinType, Set},
};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::{
    primitives::{
        babbage::{DatumHash, DatumOption},
        ToHash,
    },
    traverse::{MultiEraBlock, OutputRef},
};

use crate::dsl::task_macro::*;

carp_task! {
name MultieraDatumTask;
configuration EmptyConfig;
doc "Adds datum and datum hashes";
era multiera;
dependencies [];
read [multiera_txs];
write [];
should_add_task |block, _properties| {
  block.1.txs().iter().any(|tx| {
    tx.witnesses().plutus_data().is_some() || tx.outputs().iter().any(|output| output.datum().is_some())
  })
};
execute |previous_data, task| handle_datum(
    task.db_tx,
    task.block,
    &previous_data.multiera_txs,
);
merge_result |previous_data, _result| {
};
}

async fn handle_datum(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    multiera_txs: &[TransactionModel],
) -> Result<(), DbErr> {
    let mut hash_to_tx = BTreeMap::<DatumHash, i64>::new();
    // recall: tx may contain datum hash only w/ datum only appearing in a later tx
    let mut hash_to_data =
        BTreeMap::<DatumHash, pallas::ledger::primitives::alonzo::PlutusData>::new();
    for (tx_body, cardano_transaction) in block.1.txs().iter().zip(multiera_txs) {
        if let Some(datums) = tx_body.witnesses().plutus_data() {
            for datum in datums.iter() {
                let hash = datum.to_hash();
                hash_to_tx
                    .entry(hash)
                    .or_insert_with(|| cardano_transaction.id);
                hash_to_data.entry(hash).or_insert_with(|| datum.clone());
            }
        }
        for output in tx_body.outputs().iter() {
            match output.datum().as_ref() {
                Some(DatumOption::Hash(hash)) => {
                    hash_to_tx
                        .entry(*hash)
                        .or_insert_with(|| cardano_transaction.id);
                }
                Some(DatumOption::Data(datum)) => {
                    let hash = datum.to_hash();
                    hash_to_tx
                        .entry(hash)
                        .or_insert_with(|| cardano_transaction.id);
                    hash_to_data.entry(hash).or_insert_with(|| datum.0.clone());
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
                .join(JoinType::LeftJoin, PlutusDataRelation::PlutusDataHash.def())
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
        let to_add: Vec<PlutusDataHashActiveModel> = hash_to_tx
            .keys()
            .filter(|key| !hash_to_id.contains_key(key.as_ref()))
            .map(|key| PlutusDataHashActiveModel {
                hash: Set(key.to_vec()),
                first_tx: Set(*hash_to_tx.get(key).unwrap()),
                ..Default::default()
            })
            .collect();

        if !hash_to_tx.is_empty() {
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
            PlutusData::insert_many(to_add)
                .exec_many_with_returning(db_tx)
                .await?;
        }
    }

    Ok(())
}
