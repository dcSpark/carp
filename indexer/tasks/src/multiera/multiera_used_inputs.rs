extern crate shred;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cardano_multiplatform_lib::address::{
    BaseAddress, ByronAddress, EnterpriseAddress, PointerAddress, RewardAddress,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use nameof::name_of_type;
use pallas::ledger::primitives::alonzo::{self, TransactionBodyComponent};
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    types::TxCredentialRelationValue,
    utils::TaskPerfAggregator,
};

use super::{multiera_outputs::MultieraOutputTask, relation_map::RelationMap};

#[derive(SystemData)]
pub struct Data<'a> {
    multiera_txs: Read<'a, Vec<TransactionModel>>,
    vkey_relation_map: Write<'a, RelationMap>,
    multiera_used_inputs: Write<'a, Vec<TransactionInputModel>>,
}

pub struct MultieraUsedInputTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, alonzo::Block>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, alonzo::Block> for MultieraUsedInputTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(MultieraUsedInputTask);
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

struct MultieraUsedInputTaskBuilder;
impl<'a> TaskBuilder<'a, alonzo::Block> for MultieraUsedInputTaskBuilder {
    fn get_name(&self) -> &'static str {
        MultieraUsedInputTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        MultieraUsedInputTask::DEPENDENCIES
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
        let task = MultieraUsedInputTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry { builder: &MultieraUsedInputTaskBuilder })
}

impl<'a> System<'a> for MultieraUsedInputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_input(
                self.db_tx,
                self.block,
                &bundle.multiera_txs,
                &mut bundle.vkey_relation_map,
            ))
            .unwrap();
        *bundle.multiera_used_inputs = result;

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

async fn handle_input(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
    multiera_txs: &[TransactionModel],
    vkey_relation_map: &mut RelationMap,
) -> Result<Vec<TransactionInputModel>, DbErr> {
    let mut queued_inputs = QueuedInputs::default();

    for (tx_body, cardano_transaction) in block.1.transaction_bodies.iter().zip(multiera_txs) {
        for component in tx_body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) if cardano_transaction.is_valid => {
                    queued_inputs.push((inputs, cardano_transaction.id))
                }
                TransactionBodyComponent::Collateral(inputs) if !cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    queued_inputs.push((inputs, cardano_transaction.id))
                }
                _ => (),
            };
        }
    }

    match queued_inputs.is_empty() {
        true => Ok(vec![]),
        false => {
            let outputs_for_inputs =
                crate::era_common::get_outputs_for_inputs(&queued_inputs, db_tx).await?;
            let input_to_output_map =
                crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

            add_input_relations(
                vkey_relation_map,
                &queued_inputs,
                outputs_for_inputs
                    .iter()
                    .map(|(output, _)| output)
                    .collect::<Vec<_>>()
                    .as_slice(),
                &input_to_output_map,
            );
            Ok(
                crate::era_common::insert_inputs(&queued_inputs, &input_to_output_map, db_tx)
                    .await?,
            )
        }
    }
}

pub fn add_input_relations(
    vkey_relation_map: &mut RelationMap,
    inputs: &[(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )],
    outputs: &[&TransactionOutputModel],
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, i64>>,
) {
    let mut output_to_input_tx = BTreeMap::<i64, i64>::default();
    for input_tx_pair in inputs.iter() {
        for input in input_tx_pair.0.iter() {
            match input_to_output_map.get(&input.transaction_id.to_vec()) {
                Some(entry_for_tx) => {
                    let output_id = entry_for_tx[&(input.index as i64)];
                    output_to_input_tx.insert(output_id, input_tx_pair.1);
                }
                None => {
                    println!("tx: {}", hex::encode(input.transaction_id));
                    panic!();
                }
            }
        }
    }

    outputs.iter().for_each(|&output| {
        match &cardano_multiplatform_lib::TransactionOutput::from_bytes(output.payload.clone()) {
            Ok(payload) => {
                add_input_cred_relation(
                    vkey_relation_map,
                    output_to_input_tx[&output.id],
                    &payload.address(),
                    TxCredentialRelationValue::Input,
                    TxCredentialRelationValue::InputStake,
                );
            }
            Err(_e) => {
                // https://github.com/dcSpark/cardano-multiplatform-lib/issues/61
            }
        };
    });
}

fn add_input_cred_relation(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    addr: &cardano_multiplatform_lib::address::Address,
    input_relation: TxCredentialRelationValue,
    input_stake_relation: TxCredentialRelationValue,
) {
    if let Some(base_addr) = BaseAddress::from_address(addr) {
        // Payment Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.payment_cred().to_bytes(),
                input_relation,
            );
        }

        // Stake Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.stake_cred().to_bytes(),
                input_stake_relation,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &reward_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if ByronAddress::from_address(addr).is_some() {
        // Byron address has no credentials
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &enterprise_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(addr) {
        vkey_relation_map.add_relation(tx_id, &ptr_addr.payment_cred().to_bytes(), input_relation);
    } else {
        panic!("Unexpected address type {}", hex::encode(addr.to_bytes()));
    }
}
