use crate::{genesis, SourceConfig};
use anyhow::anyhow;
use dcspark_blockchain_source::cardano::Point;
use dcspark_blockchain_source::Source;

use deps::bigdecimal::BigDecimal;
use std::{str::FromStr, sync::Arc, thread::JoinHandle};

use crate::common::CardanoEventType;
use entity::address::Relation::TransactionOutput;
use entity::sea_orm::ColumnTrait;
use entity::sea_orm::QueryFilter;
use entity::{
    prelude::{Block, BlockColumn},
    sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect},
};
use oura::model::{
    BlockRecord, Era, Event, EventData, OutputAssetRecord, TransactionRecord, TxInputRecord,
    TxOutputRecord,
};
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2c, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};
use tasks::dsl::database_task::BlockGlobalInfo;

pub struct OuraSource {
    handles: Vec<JoinHandle<()>>,
    input: StageReceiver,
}

impl OuraSource {
    pub fn new(config: SourceConfig, start_from: Vec<Point>) -> anyhow::Result<Self> {
        match config {
            SourceConfig::Oura { network, socket } => {
                let intersect = match start_from {
                    points if points.is_empty() => {
                        // we need a special intersection type when bootstrapping from genesis
                        IntersectArg::Origin
                    }
                    points => {
                        let (slot_nb, hash) = match points.last().unwrap() {
                            Point::Origin => {
                                return Err(anyhow!("Origin point is not supported here"));
                            }
                            Point::BlockHeader { slot_nb, hash } => (slot_nb, hash),
                        };
                        tracing::info!("Starting sync at block #{} ({})", slot_nb, hash,);
                        // if last block sync'd was at slot 0,
                        // that means it was the genesis block so we start from origin
                        match slot_nb {
                            0 => IntersectArg::Origin,
                            _ => IntersectArg::Fallbacks(
                                points
                                    .iter()
                                    .flat_map(|p| match p {
                                        Point::Origin => vec![],
                                        Point::BlockHeader { slot_nb, hash } => {
                                            vec![PointArg(slot_nb.clone(), hash.clone())]
                                        }
                                    })
                                    .collect(),
                            ),
                        }
                    }
                };

                let (handles, input) = oura_bootstrap(intersect, &network, socket)?;

                Ok(OuraSource { handles, input })
            }
            _ => Err(anyhow!(
                "Config {:?} is not supported as oura config",
                config
            )),
        }
    }
}

#[async_trait::async_trait]
impl dcspark_blockchain_source::Source for OuraSource {
    type Event = CardanoEventType;
    type From = Point;

    /// note: from is ignored here since oura is set up just once
    async fn pull(&mut self, from: &Self::From) -> anyhow::Result<Option<Self::Event>> {
        let input = self
            .input
            .recv()
            .map_err(|error| anyhow!("Can't fetch oura event: {:?}", error))?;

        match input.data {
            EventData::Block(block_record) => {
                let cbor = block_record
                    .cbor_hex
                    .ok_or(anyhow!("cbor is not presented"))?;
                Ok(Some(CardanoEventType::Block {
                    cbor_hex: cbor,
                    epoch: block_record.epoch,
                    epoch_slot: block_record.epoch_slot,
                    block_number: block_record.number,
                    block_hash: block_record.hash,
                    block_slot: block_record.slot,
                }))
            }
            EventData::RollBack {
                block_slot,
                block_hash,
            } => Ok(Some(CardanoEventType::RollBack {
                block_slot,
                block_hash,
            })),
            _ => Ok(None),
        }
    }
}

fn oura_bootstrap(
    intersect: IntersectArg,
    network: &str,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    let magic = MagicArg::from_str(network).map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known));

    let mapper = mapper::Config {
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2c::Config {
        address: AddressArg(BearerKind::Unix, socket),
        // address: AddressArg(BearerKind::Tcp, socket),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
        min_depth: 0,
        intersect: Some(intersect),
        retry_policy: None,
        finalize: None, // TODO: configurable
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let mut handles = Vec::new();

    tracing::info!("{}", "Attempting to connect to node...");

    let (source_handle, source_rx) = source_setup.bootstrap().map_err(|e| {
        tracing::error!("{}", e);
        anyhow!("failed to bootstrap source. Are you sure cardano-node is running?")
    })?;

    tracing::info!("{}", "Connection to node established");

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap filter"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}
