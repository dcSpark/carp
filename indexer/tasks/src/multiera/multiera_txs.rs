use std::collections::BTreeSet;

use super::multiera_block::MultieraBlockTask;
use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::era_common::transactions_from_hashes;
use crate::{dsl::default_impl::has_transaction_multiera, dsl::task_macro::*};
use entity::sea_orm::{DatabaseTransaction, QueryOrder, Set};
use pallas::ledger::primitives::alonzo::{self};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::primitives::ToHash;
use pallas::ledger::traverse::MultiEraBlock;

carp_task! {
  name MultieraTransactionTask;
  configuration ReadonlyConfig;
  doc "Adds the transactions in the block to the database";
  era multiera;
  dependencies [MultieraBlockTask];
  read [multiera_block];
  write [multiera_txs];
  should_add_task |block, _properties| {
    has_transaction_multiera(block.1)
  };
  execute |previous_data, task| handle_tx(
      task.db_tx,
      task.block,
      &previous_data.multiera_block.as_ref().unwrap(),
      task.config.readonly
  );
  merge_result |previous_data, result| {
    *previous_data.multiera_txs = result;
  };
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>>,
    database_block: &BlockModel,
    readonly: bool,
) -> Result<Vec<TransactionModel>, DbErr> {
    if readonly {
        let txs = transactions_from_hashes(
            db_tx,
            block
                .1
                .transaction_bodies
                .iter()
                .map(|tx_body| tx_body.to_hash().to_vec())
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await;
        return txs;
    }

    let txs: Vec<TransactionActiveModel> = block
        .1
        .transaction_bodies
        .iter()
        .zip(block.1.transaction_witness_sets.iter())
        .enumerate()
        .map(|(idx, (tx_body, tx_witness_set))| {
            let body_payload = tx_body.encode_fragment().unwrap();
            let body = &cardano_multiplatform_lib::TransactionBody::from_bytes(body_payload)
                .map_err(|e| {
                    panic!(
                        "{:?}\nBlock cbor: {:?}\nTransaction body cbor: {:?}\nTx hash: {:?}\n",
                        e,
                        block.0,
                        hex::encode(tx_body.encode_fragment().unwrap()),
                        hex::encode(tx_body.to_hash())
                    )
                })
                .unwrap();

            let witness_set_payload = tx_witness_set.encode_fragment().unwrap();
            let witness_set =
                &cardano_multiplatform_lib::TransactionWitnessSet::from_bytes(witness_set_payload)
                    .map_err(|e| panic!("{:?}\nBlock cbor: {:?}", e, block.0))
                    .unwrap();

            let aux_data = block
                .1
                .auxiliary_data_set
                .iter()
                .find(|(index, _)| *index as usize == idx);

            let auxiliary_data = aux_data.map(|(_, a)| {
                let auxiliary_data_payload = a.encode_fragment().unwrap();

                cardano_multiplatform_lib::metadata::AuxiliaryData::from_bytes(
                    auxiliary_data_payload,
                )
                .map_err(|e| {
                    panic!(
                        "{:?}\n{:?}\n{:?}",
                        e,
                        hex::encode(a.encode_fragment().unwrap()),
                        cardano_multiplatform_lib::Block::from_bytes(
                            hex::decode(block.0).unwrap(),
                        )
                        .map(|block| block.to_json())
                        .map_err(|_err| block.0),
                    )
                })
                .unwrap()
            });

            let mut temp_tx =
                cardano_multiplatform_lib::Transaction::new(body, witness_set, auxiliary_data);

            let mut is_valid = true;

            if let Some(ref invalid_txs) = block.1.invalid_transactions {
                is_valid = !invalid_txs.iter().any(|i| *i as usize == idx)
            }

            temp_tx.set_is_valid(is_valid);

            TransactionActiveModel {
                hash: Set(tx_body.to_hash().to_vec()),
                block_id: Set(database_block.id),
                tx_index: Set(idx as i32),
                payload: Set(temp_tx.to_bytes()),
                is_valid: Set(is_valid),
                ..Default::default()
            }
        })
        .collect();

    if !txs.is_empty() {
        let insertions = Transaction::insert_many(txs)
            .exec_many_with_returning(db_tx)
            .await?;
        Ok(insertions)
    } else {
        Ok(vec![])
    }
}
