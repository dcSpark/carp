use crate::SourceConfig;
use anyhow::anyhow;
use dcspark_blockchain_source::cardano::Point;

use std::{str::FromStr, sync::Arc, thread::JoinHandle};

use crate::common::CardanoEventType;
use crate::types::StoppableService;
use oura::model::EventData;
use oura::pipelining::SourceProvider;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, StageReceiver},
    sources::{n2c, n2n, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

pub struct OuraSource {
    handles: Vec<JoinHandle<()>>,
    input: StageReceiver,

    // cardano-node always triggers a rollback event when you connect to it
    // if all the intersection points existed, if will return the most recent point you gave it
    // to avoid this causing a rollback when applying a migration starting from an old block, we skip this rollback
    expected_rollback: Option<PointArg>,
}

impl OuraSource {
    pub fn new(
        config: SourceConfig,
        network: String,
        start_from: Vec<Point>,
    ) -> anyhow::Result<Self> {
        match config {
            SourceConfig::Oura { socket, bearer } => {
                let (intersect, rollback) = match start_from {
                    points if points.is_empty() => {
                        // we need a special intersection type when bootstrapping from genesis
                        (IntersectArg::Origin, None)
                    }
                    points => {
                        let (slot_nb, hash) = match points.last().unwrap() {
                            Point::Origin => {
                                return Err(anyhow!("Origin point is not supported here"));
                            }
                            Point::BlockHeader { slot_nb, hash } => (slot_nb, hash),
                        };
                        tracing::info!("Starting sync at block #{} ({})", slot_nb, hash,);
                        // if last block synced was at slot 0,
                        // that means it was the genesis block so we start from origin
                        match (*slot_nb).into() {
                            0u64 => (IntersectArg::Origin, None),
                            _ => {
                                let point_args: Vec<PointArg> = points
                                    .into_iter()
                                    .flat_map(|p| match p {
                                        Point::Origin => vec![],
                                        Point::BlockHeader { slot_nb, hash } => {
                                            vec![PointArg(slot_nb.into(), hash.to_string())]
                                        }
                                    })
                                    .collect();
                                let rollback = point_args.first().cloned();
                                (IntersectArg::Fallbacks(point_args), rollback)
                            }
                        }
                    }
                };

                let (handles, input) = oura_bootstrap(bearer, intersect, &network, socket)?;

                Ok(OuraSource {
                    handles,
                    input,
                    expected_rollback: rollback,
                })
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
    async fn pull(&mut self, _from: &Self::From) -> anyhow::Result<Option<Self::Event>> {
        let input = self
            .input
            .recv()
            .map_err(|error| anyhow!("Can't fetch oura event: {:?}", error))?;

        match input.data {
            EventData::Block(block_record) => {
                let cbor = block_record
                    .cbor_hex
                    .ok_or_else(|| anyhow!("cbor is not presented"))?;
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
            } => {
                if let Some(expected) = self.expected_rollback.clone() {
                    if expected.1 == *block_hash {
                        self.expected_rollback = None;
                        return Ok(None);
                    }
                };
                Ok(Some(CardanoEventType::RollBack {
                    block_slot,
                    block_hash,
                }))
            }
            _ => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl StoppableService for OuraSource {
    async fn stop(self) -> anyhow::Result<()> {
        for handle in self.handles {
            if let Err(err) = handle.join() {
                tracing::error!("Error during oura shutdown: {:?}", err);
            }
        }

        Ok(())
    }
}

fn oura_bootstrap(
    mode: BearerKind,
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

    tracing::info!("{}", "Attempting to connect to node...");
    let (source_handle, source_rx) = match mode {
        #[allow(deprecated)]
        BearerKind::Unix => {
            let source_config = n2c::Config {
                address: AddressArg(BearerKind::Unix, socket),
                magic: Some(magic),
                well_known: None,
                mapper,
                since: None,
                min_depth: 0,
                intersect: Some(intersect),
                retry_policy: None,
                finalize: None, // TODO: configurable
            };
            WithUtils::new(source_config, utils).bootstrap()
        }
        #[allow(deprecated)]
        BearerKind::Tcp => {
            let source_config = n2n::Config {
                address: AddressArg(BearerKind::Tcp, socket),
                magic: Some(magic),
                well_known: None,
                mapper,
                since: None,
                min_depth: 0,
                intersect: Some(intersect),
                retry_policy: None,
                finalize: None, // TODO: configurable
            };
            WithUtils::new(source_config, utils).bootstrap()
        }
    }
    .map_err(|e| {
        tracing::error!("{}", e);
        anyhow!("failed to bootstrap source. Are you sure cardano-node is running?")
    })?;
    tracing::info!("{}", "Connection to node established");

    let mut handles = Vec::new();
    handles.push(source_handle);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap filter"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}
