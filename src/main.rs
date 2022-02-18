use std::{str::FromStr, sync::Arc};

use anyhow::anyhow;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider},
    sources::{n2n, AddressArg, BearerKind, MagicArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use entity::{
    prelude::{Block, BlockColumn, BlockModel},
    sea_orm::{prelude::*, Database, QueryOrder, QuerySelect},
};

mod postgres_sink;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: use an environment variable before going to production
    let conn = Database::connect("postgresql://root:root@localhost:5432/cardano").await?;

    // For rollbacks
    let blocks: Vec<BlockModel> = Block::find()
        .order_by_desc(BlockColumn::Id)
        .limit(2160)
        .all(&conn)
        .await?;

    let magic = MagicArg::from_str("testnet").map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known, None));

    let mapper = mapper::Config {
        include_block_end_events: true,
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2n::Config {
        address: AddressArg(
            BearerKind::Tcp,
            "relays-new.cardano-testnet.iohkdev.io:3001".to_string(),
        ),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![
        String::from("Transaction"),
        String::from("BlockEnd"),
        String::from("Rollback"),
    ]);

    let filter_setup = selection::Config { check };

    let (source_handle, source_rx) = source_setup
        .bootstrap()
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    sink_setup.bootstrap(filter_rx).await?;

    filter_handle.join().map_err(|_| anyhow!(""))?;
    source_handle.join().map_err(|_| anyhow!(""))?;

    Ok(())
}
