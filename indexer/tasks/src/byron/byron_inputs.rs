extern crate shred;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::byron::{self, TxIn};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::sync::{Arc, Mutex};

use crate::{
    database_task::{
        BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta, TaskBuilder, TaskRegistryEntry,
    },
    utils::TaskPerfAggregator,
};

use super::byron_outputs::ByronOutputTask;

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: Read<'a, Vec<TransactionModel>>,
    byron_inputs: Write<'a, Vec<TransactionInputModel>>,
}

pub struct ByronInputTask<'a> {
    db_tx: &'a DatabaseTransaction,
    block: BlockInfo<'a, byron::Block>,
    handle: &'a tokio::runtime::Handle,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, byron::Block> for ByronInputTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(ByronInputTask);
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(ByronOutputTask)];

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

struct ByronInputTaskBuilder;
impl<'a> TaskBuilder<'a, byron::Block> for ByronInputTaskBuilder {
    fn get_name(&self) -> &'static str {
        ByronInputTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        ByronInputTask::DEPENDENCIES
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
        let task = ByronInputTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Byron(ByronTaskRegistryEntry { builder: &ByronInputTaskBuilder })
}

impl<'a> System<'a> for ByronInputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_inputs(
                self.db_tx,
                self.block,
                bundle.byron_txs.as_slice(),
            ))
            .unwrap();
        *bundle.byron_inputs = result;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_inputs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, byron::Block>,
    byron_txs: &[TransactionModel],
) -> Result<Vec<TransactionInputModel>, DbErr> {
    match &block.1 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            let all_inputs: Vec<(
                Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
                i64,
            )> = main_block
                .body
                .tx_payload
                .iter()
                .zip(byron_txs)
                .map(|(tx_payload, cardano_tx_in_db)| {
                    let inputs: Vec<pallas::ledger::primitives::alonzo::TransactionInput> =
                        tx_payload
                            .transaction
                            .inputs
                            .iter()
                            .map(byron_input_to_alonzo)
                            .collect();
                    (inputs, cardano_tx_in_db.id)
                })
                .collect();

            let flattened_inputs: Vec<(
                &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
                i64,
            )> = all_inputs
                .iter()
                .map(|inputs| (&inputs.0, inputs.1))
                .collect();
            let outputs_for_inputs =
                crate::era_common::get_outputs_for_inputs(&flattened_inputs, db_tx).await?;

            let input_to_output_map =
                crate::era_common::gen_input_to_output_map(&outputs_for_inputs);
            let result =
                crate::era_common::insert_inputs(&flattened_inputs, &input_to_output_map, db_tx)
                    .await?;
            Ok(result)
        }
    }
}

fn byron_input_to_alonzo(input: &TxIn) -> pallas::ledger::primitives::alonzo::TransactionInput {
    match input {
        TxIn::Variant0(wrapped) => pallas::ledger::primitives::alonzo::TransactionInput {
            transaction_id: wrapped.0 .0,
            index: wrapped.0 .1 as u64,
        },
        TxIn::Other(index, tx_hash) => {
            // Note: Pallas uses "other" to future proof itself against changes in the binary spec
            todo!("handle TxIn::Other({:?}, {:?})", index, tx_hash)
        }
    }
}
