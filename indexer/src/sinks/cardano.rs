use crate::common::CardanoEventType;
use crate::perf_aggregator::PerfAggregator;
use crate::sink::Sink;
use crate::types::{MultiEraBlock, StoppableService};
use crate::{genesis, DbConfig, SinkConfig};
use anyhow::anyhow;
use async_trait::async_trait;

use dcspark_blockchain_source::cardano::Point;
use dcspark_core::{BlockId, SlotNumber};
use entity::sea_orm::Database;
use entity::sea_orm::QueryFilter;
use entity::{
    block::EraValue,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, TransactionTrait},
};
use entity::{
    prelude::{Block, BlockColumn},
    sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect},
};
use std::sync::Arc;
use std::sync::Mutex;
use tasks::byron::byron_executor::process_byron_block;
use tasks::dsl::database_task::BlockGlobalInfo;
use tasks::execution_plan::ExecutionPlan;
use tasks::multiera::multiera_executor::process_multiera_block;
use tasks::utils::TaskPerfAggregator;

pub struct CardanoSink {
    db: DatabaseConnection,
    network: String,
    genesis_folder: Option<String>,
    exec_plan: Arc<ExecutionPlan>,

    last_epoch: i128,
    epoch_start_time: std::time::Instant,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl CardanoSink {
    #[allow(unreachable_patterns)]
    pub async fn new(config: SinkConfig, exec_plan: Arc<ExecutionPlan>) -> anyhow::Result<Self> {
        let (db_config, network, genesis_folder) = match config {
            SinkConfig::Cardano {
                db,
                network,
                genesis_folder,
            } => (db, network, genesis_folder),
            _ => todo!("Invalid sink config provided"),
        };
        match db_config {
            DbConfig::Postgres { database_url } => {
                let conn = Database::connect(&database_url).await?;

                Ok(Self {
                    db: conn,
                    network,
                    genesis_folder,
                    exec_plan,
                    last_epoch: -1,
                    epoch_start_time: std::time::Instant::now(),
                    task_perf_aggregator: Arc::new(Mutex::new(TaskPerfAggregator::default())),
                })
            }
            _ => todo!("Only postgres is supported atm"),
        }
    }

    /// note: points are sorted from newest to oldest
    pub(crate) async fn get_latest_point(&self) -> anyhow::Result<Vec<Point>> {
        self.get_latest_points(1u64).await
    }

    /// note: points are sorted from newest to oldest
    pub(crate) async fn get_latest_points(&self, count: u64) -> anyhow::Result<Vec<Point>> {
        let points: Vec<Point> = Block::find()
            .order_by_desc(BlockColumn::Id)
            .limit(count)
            .all(&self.db)
            .await?
            .iter()
            .map(|block| Point::BlockHeader {
                slot_nb: SlotNumber::new(block.slot as u64),
                hash: BlockId::new(hex::encode(&block.hash)),
            })
            .collect();

        Ok(points)
        // SELECT * FROM "Block" WHERE "Block".era = 1 ORDER BY "Block".id ASC LIMIT 1;
        // for mainnet
        // start of Shelley: aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de
        // start of Allegra: 078d102d0247463f91eef69fc77f3fbbf120f3118e68cd5e6a493c15446dbf8c
        // start of Mary: a650a3f398ba4a9427ec8c293e9f7156d81fd2f7ca849014d8d2c1156c359b3a
        // start of Alonzo: 8959c0323b94cc670afe44222ab8b4e72cfcad3b5ab665f334bbe642dc6e9ef4
    }

    async fn get_specific_point(&self, block_hash: &String) -> anyhow::Result<Vec<Point>> {
        let provided_point = Block::find()
            .filter(BlockColumn::Hash.eq(hex::decode(block_hash).unwrap()))
            .one(&self.db)
            .await?;

        if provided_point.is_none() {
            panic!("Block not found in database: {block_hash}");
        }

        // for the intersection, we need to provide the block BEFORE the one the user passed in
        // since for cardano-node the block represents the last known point
        // so it will start after the point passed in

        // note: may be empty is user passed in genesis block hash
        let points: Vec<Point> = Block::find()
            .filter(BlockColumn::Id.lt(provided_point.unwrap().id))
            .order_by_desc(BlockColumn::Id)
            .one(&self.db)
            .await?
            .iter()
            .map(|block| Point::BlockHeader {
                slot_nb: SlotNumber::new(block.slot as u64),
                hash: BlockId::new(hex::encode(&block.hash)),
            })
            .collect();

        Ok(points)
    }
}

const KNOWN_GENESIS_FOLDER: &str = "./genesis";

#[async_trait]
impl Sink for CardanoSink {
    type From = Point;
    type Event = CardanoEventType;

    async fn start_from(&mut self, from: Option<String>) -> anyhow::Result<Vec<Self::From>> {
        let start = match &from {
            None => self.get_latest_point().await?,
            Some(block) => self.get_specific_point(block).await?,
        };

        if start.is_empty() {
            // https://github.com/txpipe/oura/blob/67b01e8739ed2927ced270e08daea74b03bcc7f7/src/sources/common.rs#L91
            let genesis_folder: &str = match dbg!(&self.network[..]) {
                "mainnet" | "testnet" | "preview" | "preprod" => KNOWN_GENESIS_FOLDER,
                "custom" => &self
                    .genesis_folder
                    .as_ref()
                    .expect("genesis_folder should be specified for custom networks")[..],
                rest => {
                    return Err(anyhow!(
                        "{} is invalid. NETWORK must be either mainnet/preview/preprod/testnet or a 'custom' network",
                        rest
                    ))
                }
            };
            genesis::process_genesis(
                &self.db,
                &self.network,
                genesis_folder,
                self.exec_plan.clone(),
            )
            .await?;
            return self.get_latest_point().await;
        }

        Ok(start)
    }

    async fn process(
        &mut self,
        event: Self::Event,
        perf_aggregator: &mut PerfAggregator,
    ) -> anyhow::Result<()> {
        match event {
            CardanoEventType::Block {
                cbor_hex,
                epoch,
                epoch_slot,
                block_number,
                block_hash,
                block_slot: _block_slot,
            } => {
                match epoch {
                    Some(epoch) if epoch as i128 > self.last_epoch => {
                        let epoch_duration = self.epoch_start_time.elapsed();
                        perf_aggregator.set_overhead(
                            &epoch_duration,
                            &self.task_perf_aggregator.lock().unwrap().get_total(),
                        );

                        // skip posting stats if last_epoch == -1 (went application just launched)
                        if self.last_epoch >= 0 {
                            tracing::info!(
                                "Finished processing epoch {} after {:?}s (+{:?}s)",
                                self.last_epoch,
                                epoch_duration
                                    .checked_sub(perf_aggregator.block_fetch)
                                    .unwrap_or(std::time::Duration::new(0, 0))
                                    .as_secs(),
                                perf_aggregator.block_fetch.as_secs()
                            );

                            tracing::trace!(
                                    "Epoch non-task time spent:\n{:#?}\nEpoch task-wise time spent:\n{:#?}",
                                    perf_aggregator,
                                    self.task_perf_aggregator.lock().unwrap()
                                );
                        }
                        self.epoch_start_time = std::time::Instant::now();
                        perf_aggregator.reset();
                        self.task_perf_aggregator =
                            Arc::new(Mutex::new(TaskPerfAggregator::default()));

                        tracing::info!(
                            "Starting epoch {} at block #{} ({})",
                            epoch,
                            block_number,
                            block_hash
                        );
                        self.last_epoch = epoch as i128;
                    }
                    _ => (),
                };
                self.db
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(insert_block(
                            cbor_hex,
                            epoch,
                            epoch_slot,
                            txn,
                            self.exec_plan.clone(),
                            self.task_perf_aggregator.clone(),
                        ))
                    })
                    .await?;
            }
            CardanoEventType::RollBack {
                block_slot,
                block_hash,
            } => {
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
                    .one(&self.db)
                    .await?;
                match &point {
                    None => {
                        // note: potentially caused by https://github.com/txpipe/oura/issues/304
                        let count = Block::find().count(&self.db).await?;
                        if count > 1 {
                            panic!(
                                "Rollback destination did not exist. Maybe you're stuck on a fork?"
                            );
                        }
                    }
                    Some(point) => {
                        Block::delete_many()
                            .filter(BlockColumn::Id.gt(point.id))
                            .exec(&self.db)
                            .await?;
                    }
                }

                perf_aggregator.rollback += rollback_start.elapsed();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl StoppableService for CardanoSink {
    async fn stop(self) -> anyhow::Result<()> {
        Ok(())
    }
}

fn to_era_value(x: &MultiEraBlock) -> EraValue {
    match x {
        MultiEraBlock::Byron(_) => EraValue::Byron,
        MultiEraBlock::Shelley(_) => EraValue::Shelley,
        MultiEraBlock::Allegra(_) => EraValue::Allegra,
        MultiEraBlock::Mary(_) => EraValue::Mary,
        MultiEraBlock::Alonzo(_) => EraValue::Alonzo,
        MultiEraBlock::Babbage(_) => EraValue::Babbage,
        _ => unreachable!("all known eras are handled"),
    }
}

async fn insert_block(
    cbor_hex: String,
    epoch: Option<u64>,
    epoch_slot: Option<u64>,
    txn: &DatabaseTransaction,
    exec_plan: Arc<ExecutionPlan>,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let mut perf_aggregator = PerfAggregator::new();

    let block_parse_counter = std::time::Instant::now();

    let block_payload = hex::decode(cbor_hex.clone()).unwrap();
    let multi_block = MultiEraBlock::from_explicit_network_cbor_bytes(&block_payload).unwrap();

    let block_global_info = BlockGlobalInfo {
        era: to_era_value(&multi_block),
        epoch,
        epoch_slot,
    };

    perf_aggregator.block_parse += block_parse_counter.elapsed();

    match &multi_block {
        MultiEraBlock::Byron(_byron) => {
            process_byron_block(
                txn,
                (&cbor_hex, &multi_block, &block_global_info),
                &exec_plan,
                task_perf_aggregator.clone(),
            )
            .await?
        }
        _ => {
            process_multiera_block(
                txn,
                (&cbor_hex, &multi_block, &block_global_info),
                &exec_plan,
                task_perf_aggregator.clone(),
            )
            .await?
        }
    }

    Ok(())
}
