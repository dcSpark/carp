extern crate shred;

use std::sync::{Arc, Mutex};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use nameof::name_of_type;
use pallas::ledger::primitives::{byron, Fragment};
use shred::{DispatcherBuilder, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta, TaskBuilder, TaskRegistryEntry,
    },
    utils::{blake2b256, TaskPerfAggregator},
};

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: Write<'a, Vec<TransactionModel>>,
}

pub struct ByronTransactionTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, byron::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, byron::Block> for ByronTransactionTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(ByronTransactionTask);
    const DEPENDENCIES: &'static [&'static str] = &[];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, byron::Block>,
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

struct ByronTransactionTaskBuilder;
impl<'a> TaskBuilder<'a, byron::Block> for ByronTransactionTaskBuilder {
    fn get_name(&self) -> &'static str {
        ByronTransactionTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        ByronTransactionTask::DEPENDENCIES
    }

    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, byron::Block>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        _properties: &ini::Properties,
    ) {
        let task = ByronTransactionTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Byron(ByronTaskRegistryEntry { builder: &ByronTransactionTaskBuilder })
}

impl<'a> System<'a> for ByronTransactionTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_tx(self.db_tx, self.block))
            .unwrap();
        *bundle.byron_txs = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_tx(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
) -> Result<Vec<TransactionModel>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            if main_block.body.tx_payload.is_empty() {
                return Ok(vec![]);
            }

            let transaction_inserts =
                Transaction::insert_many(main_block.body.tx_payload.iter().enumerate().map(
                    |(idx, tx_body)| {
                        let tx_hash = blake2b256(&tx_body.transaction.encode_fragment().expect(""));

                        let tx_payload = tx_body.encode_fragment().unwrap();

                        TransactionActiveModel {
                            hash: Set(tx_hash.to_vec()),
                            block_id: Set(block.2.id),
                            tx_index: Set(idx as i32),
                            payload: Set(tx_payload),
                            is_valid: Set(true), // always true in Byron
                            ..Default::default()
                        }
                    },
                ))
                .exec_many_with_returning(db_tx)
                .await?;
            Ok(transaction_inserts)
        }
    }
}
