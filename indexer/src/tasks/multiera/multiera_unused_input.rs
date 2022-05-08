extern crate shred;

use std::sync::{Arc, Mutex};

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    relation_map::RelationMap,
    tasks::{
        database_task::{
            BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
        },
        utils::TaskPerfAggregator,
    },
};

use super::{multiera_outputs::MultieraOutputTask, multiera_used_inputs::add_input_relations};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    vkey_relation_map: Write<'a, RelationMap>,
}

pub struct MultieraUnusedInputTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraUnusedInputTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraUnusedInputTask);
    // note: inputs have to be added AFTER outputs added to DB
    const DEPENDENCIES: &'static [&'static str] = &[name_of_type!(MultieraOutputTask)];

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

struct MultieraUnusedInputTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraUnusedInputTaskBuilder {
    fn get_name() -> &'static str {
        MultieraUnusedInputTask::TASK_NAME
    }
    fn get_dependencies() -> &'static [&'static str] {
        MultieraUnusedInputTask::DEPENDENCIES
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
        let task = MultieraUnusedInputTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, Self::get_name(), Self::get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry {name: MultieraUnusedInputTask::TASK_NAME, builder: &MultieraUnusedInputTaskBuilder })
}

impl<'a> System<'a> for MultieraUnusedInputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        self.handle
            .block_on(handle_unused_input(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &mut bundle.vkey_relation_map,
            ))
            .unwrap();

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

type QueuedInputs<'a> = Vec<(
    &'a Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
    i64,
)>;

async fn handle_unused_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<(), DbErr> {
    let mut queued_unused_inputs = QueuedInputs::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) if !cardano_transaction.is_valid => {
                    queued_unused_inputs.push((inputs, cardano_transaction.id))
                }
                TransactionBodyComponent::Collateral(inputs) if cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    queued_unused_inputs.push((inputs, cardano_transaction.id))
                }
                _ => (),
            };
        }
    }

    if !queued_unused_inputs.is_empty() {
        let outputs_for_inputs =
            crate::tasks::era_common::get_outputs_for_inputs(&queued_unused_inputs, db_tx).await?;
        let input_to_output_map =
            crate::tasks::era_common::gen_input_to_output_map(&outputs_for_inputs);

        add_input_relations(
            vkey_relation_map,
            &queued_unused_inputs,
            outputs_for_inputs
                .iter()
                .map(|(output, _)| output)
                .collect::<Vec<_>>()
                .as_slice(),
            &input_to_output_map,
        );
    }

    Ok(())
}
