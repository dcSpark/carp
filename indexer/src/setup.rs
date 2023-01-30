use std::{str::FromStr, sync::Arc, thread::JoinHandle};

use anyhow::anyhow;

use entity::sea_orm::ColumnTrait;
use entity::sea_orm::QueryFilter;
use entity::{
    prelude::{Block, BlockColumn},
    sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect},
};
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2c, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

/// note: points are sorted from newest to oldest
pub async fn get_latest_points(conn: &DatabaseConnection) -> anyhow::Result<Vec<PointArg>> {
    let points: Vec<PointArg> = Block::find()
        .order_by_desc(BlockColumn::Id)
        .limit(1)
        .all(conn)
        .await?
        .iter()
        .map(|block| PointArg(block.slot as u64, hex::encode(&block.hash)))
        .collect();

    Ok(points)
    // SELECT * FROM "Block" WHERE "Block".era = 1 ORDER BY "Block".id ASC LIMIT 1;
    // for mainnet
    // start of Shelley: aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de
    // start of Allegra: 078d102d0247463f91eef69fc77f3fbbf120f3118e68cd5e6a493c15446dbf8c
    // start of Mary: a650a3f398ba4a9427ec8c293e9f7156d81fd2f7ca849014d8d2c1156c359b3a
    // start of Alonzo: 8959c0323b94cc670afe44222ab8b4e72cfcad3b5ab665f334bbe642dc6e9ef4
}

pub async fn get_specific_point(
    conn: &DatabaseConnection,
    block_hash: &str,
) -> anyhow::Result<Vec<PointArg>> {
    let provided_point = Block::find()
        .filter(BlockColumn::Hash.eq(hex::decode(block_hash).unwrap()))
        .one(conn)
        .await?;

    if provided_point.is_none() {
        panic!("Block not found in database: {block_hash}");
    }

    // for the intersection, we need to provide the block BEFORE the one the user passed in
    // since for cardano-node the block represents the last known point
    // so it will start after the point passed in

    // note: may be empty is user passed in genesis block hash
    let points: Vec<PointArg> = Block::find()
        .filter(BlockColumn::Id.lt(provided_point.unwrap().id))
        .order_by_desc(BlockColumn::Id)
        .one(conn)
        .await?
        .iter()
        .map(|block| PointArg(block.slot as u64, hex::encode(&block.hash)))
        .collect();

    Ok(points)
}

pub fn oura_bootstrap(
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
