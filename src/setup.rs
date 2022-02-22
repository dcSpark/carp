use std::{str::FromStr, sync::Arc, thread::JoinHandle};

use anyhow::anyhow;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2n, AddressArg, BearerKind, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use entity::{
    prelude::{Block, BlockColumn},
    sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect},
};

pub async fn get_latest_points(conn: &DatabaseConnection) -> anyhow::Result<Vec<PointArg>> {
    let points: Vec<PointArg> = Block::find()
        .order_by_desc(BlockColumn::Id)
        .limit(2160)
        .all(conn)
        .await?
        .iter()
        .map(|block| PointArg(block.slot as u64, hex::encode(&block.hash)))
        .collect();

    Ok(points)
}

pub fn oura_bootstrap(
    points: Vec<PointArg>,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    let intersections = if !points.is_empty() {
        Some(points)
    } else {
        Some(vec![PointArg(
            0,
            "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f".to_string(),
        )])
    };

    let magic = MagicArg::from_str("mainnet").map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known));

    let mapper = mapper::Config {
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2n::Config {
        address: AddressArg(
            BearerKind::Tcp,
            "relays-new.cardano-mainnet.iohk.io:3001".to_string(),
        ),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
        min_depth: 0,
        intersections,
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let mut handles = Vec::new();

    let (source_handle, source_rx) = source_setup
        .bootstrap()
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}
