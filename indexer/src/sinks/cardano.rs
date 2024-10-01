use crate::common::CardanoEventType;
use crate::perf_aggregator::PerfAggregator;
use crate::sink::Sink;
use crate::types::{MultiEraBlock, StoppableService};
use crate::{genesis, DbConfig, SinkConfig};
use anyhow::{anyhow, Context as _};
use async_trait::async_trait;
use dcspark_blockchain_source::cardano::time::Era;
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
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tasks::byron::byron_executor::process_byron_block;
use tasks::dsl::database_task::BlockGlobalInfo;
use tasks::execution_plan::ExecutionPlan;
use tasks::multiera::multiera_executor::process_multiera_block;
use tasks::utils::TaskPerfAggregator;

#[derive(Clone)]
pub enum Network {
    Mainnet,
    Preview,
    Preprod,
    Sanchonet,
    Custom { genesis_files: PathBuf },
}

impl Network {
    pub fn genesis_filename(&self, era: EraValue) -> String {
        match self {
            Network::Mainnet | Network::Preview | Network::Preprod | Network::Sanchonet => {
                format!("{}-{}-genesis.json", self.to_str(), era.to_str())
            }
            Network::Custom { genesis_files: _ } => format!("{}-genesis.json", era.to_str()),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Preview => "preview",
            Network::Preprod => "preprod",
            Network::Sanchonet => "sanchonet",
            Network::Custom { genesis_files: _ } => "custom",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

pub struct CardanoSink {
    db: DatabaseConnection,
    network: Network,
    exec_plan: Arc<ExecutionPlan>,

    last_epoch: i128,
    epoch_start_time: std::time::Instant,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    shelley_era: Option<Era>,
}

impl CardanoSink {
    #[allow(unreachable_patterns)]
    pub async fn new(config: SinkConfig, exec_plan: Arc<ExecutionPlan>) -> anyhow::Result<Self> {
        let (db_config, network, genesis_folder) = match config {
            SinkConfig::Cardano {
                db,
                network,
                custom_config: _,
                genesis_folder,
            } => (db, network, genesis_folder),
            _ => todo!("Invalid sink config provided"),
        };

        let network = if network == "custom" {
            Network::Custom {
                genesis_files: genesis_folder
                    .ok_or(anyhow!(
                        "genesis_folder should be specified for custom networks"
                    ))?
                    .into(),
            }
        } else {
            match network.as_ref() {
                "mainnet" => Network::Mainnet,
                "preview" => Network::Preview,
                "preprod" => Network::Preprod,
                "sanchonet" => Network::Sanchonet,
                unknown => {
                    anyhow::bail!(
                            "{unknown} is invalid. NETWORK must be either mainnet/preview/preprod or a 'custom' network",
                )
                }
            }
        };

        match db_config {
            DbConfig::Postgres { database_url } => {
                let conn = Database::connect(&database_url).await?;

                let shelley_era = get_shelley_era_data_from_db(&conn).await?;

                Ok(Self {
                    db: conn,
                    network,
                    exec_plan,
                    last_epoch: -1,
                    epoch_start_time: std::time::Instant::now(),
                    task_perf_aggregator: Arc::new(Mutex::new(TaskPerfAggregator::default())),
                    shelley_era,
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
            let genesis_file: PathBuf = get_genesis_file(&self.network, EraValue::Byron)?;
            genesis::process_byron_genesis(
                &self.db,
                &genesis_file.to_string_lossy(),
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
                mut epoch,
                epoch_slot,
                block_number,
                block_hash,
                block_slot: _block_slot,
            } => {
                // this won't work for the first block in the shelley era, since
                // the shelley genesis is processed after this however, this
                // probably doesn't matter that much for the perf aggregator
                // since it's only one block and it only happens once.
                if let Some(shelley_era) = &self.shelley_era {
                    epoch =
                        epoch.or_else(|| shelley_era.absolute_slot_to_epoch(epoch_slot.unwrap()));
                }

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
                self.shelley_era = self
                    .db
                    .transaction::<_, Option<Era>, DbErr>(|txn| {
                        Box::pin(insert_block(
                            cbor_hex,
                            epoch,
                            epoch_slot,
                            txn,
                            self.exec_plan.clone(),
                            self.task_perf_aggregator.clone(),
                            self.network.clone(),
                            self.shelley_era.clone(),
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

                        // the table that keeps track of the shelley genesis
                        // has a foreign key to the block in which we triggered
                        // that update, this means the entry will get deleted if
                        // we rollback to a point before that, in which case we
                        // re-fetch it just to be sure.
                        self.shelley_era = get_shelley_era_data_from_db(&self.db).await?;
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
        MultiEraBlock::Conway(_) => EraValue::Conway,
    }
}

#[allow(clippy::too_many_arguments)]
async fn insert_block(
    cbor_hex: String,
    epoch: Option<u64>,
    epoch_slot: Option<u64>,
    txn: &DatabaseTransaction,
    exec_plan: Arc<ExecutionPlan>,
    task_perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    network: Network,
    mut shelley_era: Option<Era>,
) -> Result<Option<Era>, DbErr> {
    let mut perf_aggregator = PerfAggregator::new();

    let block_parse_counter = std::time::Instant::now();

    let block_payload = hex::decode(cbor_hex.clone()).unwrap();
    let multi_block = MultiEraBlock::from_explicit_network_cbor_bytes(&block_payload).unwrap();

    let era = to_era_value(&multi_block);
    let mut block_global_info = BlockGlobalInfo {
        era,
        epoch,
        epoch_slot,
    };

    if era > EraValue::Byron && shelley_era.is_none() {
        // we don't have the code to parse the other genesis blocks (alonzo, conway).
        let genesis_file_path = get_genesis_file(&network, EraValue::Shelley)
            .context("Couldn't get the shelley genesis file from the filesystem")
            .unwrap();

        tasks::genesis::genesis_executor::process_shelley_genesis_block(
            txn,
            ("", &genesis_file_path, &block_global_info),
            &exec_plan,
            task_perf_aggregator.clone(),
        )
        .await?;

        shelley_era = entity::genesis::Entity::find()
            .filter(entity::genesis::Column::Era.eq(i32::from(EraValue::Shelley)))
            .limit(1)
            .one(txn)
            .await?
            .map(|model| Era {
                first_slot: model.first_slot.try_into().unwrap(),
                start_epoch: model.start_epoch.try_into().unwrap(),
                epoch_length_seconds: model.epoch_length_seconds.try_into().unwrap(),
                // we don't need to know these since we don't compute timestamps
                known_time: 0,
                slot_length: 0,
            });

        // in the byron era the epoch it's in the header, so we only need to compute
        // this if we already transitioned to shelley.
        if let Some(shelley_era) = &shelley_era {
            block_global_info.epoch = block_global_info.epoch.or_else(|| {
                shelley_era.absolute_slot_to_epoch(block_global_info.epoch_slot.unwrap())
            });
        }
    }

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

    Ok(shelley_era)
}

fn get_genesis_file(network: &Network, era: EraValue) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::new();

    let known_genesis_folder = PathBuf::from(KNOWN_GENESIS_FOLDER);
    let genesis_folder = match network {
        Network::Mainnet | Network::Preview | Network::Preprod | Network::Sanchonet => {
            &known_genesis_folder
        }
        Network::Custom { genesis_files } => genesis_files,
    };

    path.push(genesis_folder);
    path.push(network.genesis_filename(era));

    Ok(path)
}

async fn get_shelley_era_data_from_db(
    conn: &DatabaseConnection,
) -> Result<Option<Era>, anyhow::Error> {
    let shelley_era = entity::genesis::Entity::find()
        .filter(entity::genesis::Column::Era.eq(i32::from(EraValue::Shelley)))
        .limit(1)
        .one(conn)
        .await?
        .map(|model| {
            Era {
                first_slot: model.first_slot.try_into().unwrap(),
                start_epoch: model.start_epoch.try_into().unwrap(),
                epoch_length_seconds: model.epoch_length_seconds.try_into().unwrap(),
                // we don't need to know these since we don't compute timestamps
                known_time: 0,
                slot_length: 0,
            }
        });
    Ok(shelley_era)
}
