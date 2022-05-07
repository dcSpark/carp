use anyhow::anyhow;
use oura::{
    model::{BlockRecord, Era, EventData},
    pipelining::StageReceiver,
};
use pallas::ledger::primitives::{
    alonzo::{self},
    Fragment,
};

use crate::types::MultiEraBlock;
use crate::{perf_aggregator::PerfAggregator, types::EraValue};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, Set, TransactionTrait},
};
use migration::DbErr;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

impl<'a> Config<'a> {
    pub async fn bootstrap(&self, input: StageReceiver) -> anyhow::Result<()> {
        tracing::info!("{}", "Starting to process blocks");

        let mut last_epoch: i128 = -1;
        let mut epoch_start_time = std::time::Instant::now();
        let mut perf_aggregator = PerfAggregator::new();

        loop {
            let event_fetch_start = std::time::Instant::now();
            let event = input.recv()?;
            perf_aggregator.block_fetch += event_fetch_start.elapsed();

            match &event.data {
                EventData::Block(block_record) => {
                    match block_record.epoch {
                        Some(epoch) if epoch as i128 > last_epoch => {
                            let epoch_duration = epoch_start_time.elapsed();
                            perf_aggregator.set_overhead(&epoch_duration);

                            // skip posting stats if last_epoch == -1 (went application just launched)
                            if last_epoch >= 0 {
                                tracing::info!(
                                    "Finished processing epoch {} after {:?} (+{:?})",
                                    last_epoch,
                                    epoch_duration
                                        .checked_sub(perf_aggregator.block_fetch)
                                        .unwrap_or(std::time::Duration::new(0, 0)),
                                    perf_aggregator.block_fetch
                                );

                                tracing::trace!(
                                    "Epoch part-wise time spent:\n{:#?}",
                                    perf_aggregator
                                );
                            }
                            epoch_start_time = std::time::Instant::now();
                            perf_aggregator = PerfAggregator::new();

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
                    perf_aggregator += self
                        .conn
                        .transaction::<_, PerfAggregator, DbErr>(|txn| {
                            Box::pin(insert_block(block_record.clone(), txn))
                        })
                        .await?;
                }
                EventData::RollBack { block_slot, .. } => {
                    match block_slot {
                        0 => tracing::info!("Rolling back to genesis"),
                        _ => tracing::info!("Rolling back to slot {}", block_slot - 1),
                    };
                    let rollback_start = std::time::Instant::now();
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gt(*block_slot))
                        .exec(self.conn)
                        .await?;
                    perf_aggregator.rollback += rollback_start.elapsed();
                }
                _ => (),
            }
        }
    }
}

async fn insert_block(
    block_record: BlockRecord,
    txn: &DatabaseTransaction,
) -> Result<PerfAggregator, DbErr> {
    println!("Start insert block");
    let mut perf_aggregator = PerfAggregator::new();
    let mut time_counter = std::time::Instant::now();

    let hash = hex::decode(&block_record.hash).unwrap();
    let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();

    perf_aggregator.block_parse += time_counter.elapsed();
    time_counter = std::time::Instant::now();

    let (multi_block, era) = block_with_era(block_record.era, &block_payload).unwrap();

    let block = BlockActiveModel {
        era: Set(era.into()),
        hash: Set(hash),
        height: Set(block_record.number as i32),
        epoch: Set(block_record.epoch.unwrap() as i32),
        slot: Set(block_record.slot as i32),
        ..Default::default()
    };

    let block = block.insert(txn).await?;

    perf_aggregator.block_insertion += time_counter.elapsed();
    time_counter = std::time::Instant::now();

    match &multi_block {
        MultiEraBlock::Byron(byron_block) => {
            crate::byron::process_byron_block(
                &mut perf_aggregator,
                &mut time_counter,
                txn,
                (byron_block, &block),
            )
            .await?
        }
        MultiEraBlock::Compatible(alonzo_block) => {
            crate::multiera::process_multiera_block(
                &mut perf_aggregator,
                &mut time_counter,
                txn,
                &block_record,
                &block,
                alonzo_block,
            )
            .await?
        }
    }

    Ok(perf_aggregator)
}

fn block_with_era(era: Era, payload: &[u8]) -> anyhow::Result<(MultiEraBlock, EraValue)> {
    let data = match era {
        Era::Byron => {
            let block = pallas::ledger::primitives::byron::Block::decode_fragment(payload)
                .map_err(|_| anyhow!("failed to decode cbor"))?;

            (MultiEraBlock::Byron(Box::new(block)), EraValue::Byron)
        }
        rest => {
            let alonzo::BlockWrapper(_, block) = alonzo::BlockWrapper::decode_fragment(payload)
                .map_err(|_| anyhow!("failed to decode cbor"))?;

            let box_block = Box::new(block);

            match rest {
                Era::Shelley => (MultiEraBlock::Compatible(box_block), EraValue::Shelley),
                Era::Allegra => (MultiEraBlock::Compatible(box_block), EraValue::Allegra),
                Era::Mary => (MultiEraBlock::Compatible(box_block), EraValue::Mary),
                Era::Alonzo => (MultiEraBlock::Compatible(box_block), EraValue::Alonzo),
                _ => unreachable!(),
            }
        }
    };

    Ok(data)
}
