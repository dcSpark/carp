extern crate shred;

use std::sync::{Arc, Mutex};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self};
use pallas::ledger::primitives::Fragment;
use shred::{DispatcherBuilder, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    utils::TaskPerfAggregator,
};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Write<'a, Vec<TransactionModel>>,
}

pub struct MultieraTransactionTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraTransactionTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraTransactionTask);
    const DEPENDENCIES: &'static [&'static str] = &[];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, alonzo::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Self {
        Self {
            db_tx,
            block,
            handle,
            perf_aggregator,
        }
    }
}

struct MultieraTransactionTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraTransactionTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraTransactionTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraTransactionTask::DEPENDENCIES
    }

    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, alonzo::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        _properties: &ini::Properties,
    ) {
        let task = MultieraTransactionTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraTransactionTaskBuilder })
}

impl<'a> System<'a> for MultieraTransactionTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_tx(self.db_tx, self.block))
            .unwrap();
        *bundle.multiera_txs = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
) -> Result<Vec<TransactionModel>, DbErr> {
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
                block_id: Set(block.2.id),
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
