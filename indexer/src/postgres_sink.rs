use oura::{
    model::{BlockRecord, EventData},
    pipelining::StageReceiver,
    sources::PointArg,
};
use std::sync::{Arc, Mutex};
use tasks::{
    byron::byron_executor::process_byron_block, dsl::database_task::BlockGlobalInfo,
    execution_plan::ExecutionPlan, multiera::multiera_executor::process_multiera_block,
    utils::TaskPerfAggregator,
};

use crate::perf_aggregator::PerfAggregator;
use crate::types::MultiEraBlock;
use entity::{
    block::EraValue,
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, TransactionTrait},
};
use migration::DbErr;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

impl<'a> Config<'a> {
    pub async fn start(
        &self,
        input: StageReceiver,
        exec_plan: Arc<ExecutionPlan>,
        initial_point: Option<&PointArg>,
    ) -> anyhow::Result<()> {
        tracing::info!("{}", "Starting to process blocks");
        let mut expected_rollback = initial_point;

        let mut last_epoch: i128 = -1;
        let mut epoch_start_time = std::time::Instant::now();
        let mut perf_aggregator = PerfAggregator::new();
        let mut task_perf_aggregator = Arc::new(Mutex::new(TaskPerfAggregator::default()));

        loop {
            let event_fetch_start = std::time::Instant::now();
            let event = input.recv()?;
            perf_aggregator.block_fetch += event_fetch_start.elapsed();

            match &event.data {
                EventData::Block(block_record) => {
                    match block_record.epoch {
                        Some(epoch) if epoch as i128 > last_epoch => {
                            let epoch_duration = epoch_start_time.elapsed();
                            perf_aggregator.set_overhead(
                                &epoch_duration,
                                &task_perf_aggregator.lock().unwrap().get_total(),
                            );

                            // skip posting stats if last_epoch == -1 (went application just launched)
                            if last_epoch >= 0 {
                                tracing::info!(
                                    "Finished processing epoch {} after {:?}s (+{:?}s)",
                                    last_epoch,
                                    epoch_duration
                                        .checked_sub(perf_aggregator.block_fetch)
                                        .unwrap_or(std::time::Duration::new(0, 0))
                                        .as_secs(),
                                    perf_aggregator.block_fetch.as_secs()
                                );

                                tracing::trace!(
                                    "Epoch non-task time spent:\n{:#?}\nEpoch task-wise time spent:\n{:#?}",
                                    perf_aggregator,
                                    task_perf_aggregator.lock().unwrap()
                                );
                            }
                            epoch_start_time = std::time::Instant::now();
                            perf_aggregator = PerfAggregator::new();
                            task_perf_aggregator =
                                Arc::new(Mutex::new(TaskPerfAggregator::default()));

                            tracing::info!(
                                "Starting epoch {} at block #{} ({})",
                                epoch,
                                block_record.number,
                                block_record.hash
                            );
                            last_epoch = epoch as i128;
                        }
                        _ => (),
                    };
                    self.conn
                        .transaction::<_, (), DbErr>(|txn| {
                            Box::pin(insert_block(
                                block_record.clone(),
                                txn,
                                exec_plan.clone(),
                                task_perf_aggregator.clone(),
                            ))
                        })
                        .await?;
                }
                EventData::RollBack {
                    block_slot,
                    block_hash,
                } => {
                    // cardano-node always triggers a rollback event when you connect to it
                    // if all the intersection points existed, if will return the most recent point you gave it
                    // to avoid this causing a rollback when applying a migration starting from an old block, we skip this rollback
                    if let Some(expected) = expected_rollback {
                        if expected.1 == *block_hash {
                            expected_rollback = None;
                            continue;
                        }
                    };
                    match block_slot {
                        0 => tracing::info!("Rolling back to genesis ({})", block_hash),
                        _ => tracing::info!(
                            "Rolling back to block {} at slot {}",
                            block_hash,
                            block_slot - 1
                        ),
                    };
                    let rollback_start = std::time::Instant::now();

                    let point = Block::find()
                        .filter(BlockColumn::Hash.eq(hex::decode(block_hash).unwrap()))
                        .one(self.conn)
                        .await?;
                    match &point {
                        None => {
                            // note: potentially caused by https://github.com/txpipe/oura/issues/304
                            let count = Block::find().count(self.conn).await?;
                            if count > 1 {
                                panic!(
                                "Rollback destination did not exist. Maybe you're stuck on a fork?"
                            );
                            }
                        }
                        Some(point) => {
                            Block::delete_many()
                                .filter(BlockColumn::Id.gt(point.id))
                                .exec(self.conn)
                                .await?;
                        }
                    }

                    perf_aggregator.rollback += rollback_start.elapsed();
                }
                _ => (),
            }
        }
    }
}

fn to_era_value(x: pallas::ledger::traverse::Era) -> EraValue {
    match x {
        pallas::ledger::traverse::Era::Byron => EraValue::Byron,
        pallas::ledger::traverse::Era::Shelley => EraValue::Shelley,
        pallas::ledger::traverse::Era::Allegra => EraValue::Allegra,
        pallas::ledger::traverse::Era::Mary => EraValue::Mary,
        pallas::ledger::traverse::Era::Alonzo => EraValue::Alonzo,
        pallas::ledger::traverse::Era::Babbage => EraValue::Babbage,
        _ => unreachable!("all known eras are handled"),
    }
}

async fn insert_block(
    block_record: BlockRecord,
    txn: &DatabaseTransaction,
    exec_plan: Arc<ExecutionPlan>,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let mut perf_aggregator = PerfAggregator::new();

    let block_parse_counter = std::time::Instant::now();

    let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();
    let multi_block = MultiEraBlock::decode(&block_payload).unwrap();

    let block_global_info = BlockGlobalInfo {
        era: to_era_value(multi_block.era()),
        epoch: block_record.epoch,
        epoch_slot: block_record.epoch_slot,
    };

    perf_aggregator.block_parse += block_parse_counter.elapsed();

    match &multi_block.era() {
        pallas::ledger::traverse::Era::Byron => {
            process_byron_block(
                txn,
                (
                    &block_record.cbor_hex.unwrap(),
                    &multi_block,
                    &block_global_info,
                ),
                &exec_plan,
                task_perf_aggregator.clone(),
            )
            .await?
        }
        _ => {
            process_multiera_block(
                txn,
                (
                    &block_record.cbor_hex.unwrap(),
                    &multi_block,
                    &block_global_info,
                ),
                &exec_plan,
                task_perf_aggregator.clone(),
            )
            .await?
        }
    }

    Ok(())
}
