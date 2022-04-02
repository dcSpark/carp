use std::{str::FromStr, sync::Arc, thread::JoinHandle};

use anyhow::anyhow;

use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2c, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use entity::{
    prelude::{
        AddressActiveModel, Block, BlockActiveModel, BlockColumn, TransactionActiveModel,
        TransactionOutputActiveModel,
    },
    sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, QuerySelect, Set},
};

use crate::types::GenesisFile;

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
}

pub fn oura_bootstrap(
    points: Vec<PointArg>,
    network: &str,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    // we need a special intersection type when bootstrapping from genesis
    let intersect = match points.len() {
        0 => Err(anyhow!("Missing intersection point for bootstrapping")),
        1 => Ok(IntersectArg::Origin),
        _ => Ok(IntersectArg::Fallbacks(points))
    }?;

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
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let mut handles = Vec::new();

    let (source_handle, source_rx) = source_setup.bootstrap().map_err(|e| {
        eprintln!("{}", e);
        anyhow!("failed to bootstrap source")
    })?;

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap filter"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}

const GENESIS_MAINNET: &str = include_str!("../genesis/mainnet.json");
const GENESIS_TESTNET: &str = include_str!("../genesis/testnet.json");

pub async fn insert_genesis(conn: &DatabaseConnection, network: &str) -> anyhow::Result<()> {
    let genesis_str = match network {
        "mainnet" => GENESIS_MAINNET,
        "testnet" => GENESIS_TESTNET,
        rest => {
            return Err(anyhow!(
                "{} is invalid. NETWORK must be either mainnet or testnet",
                rest
            ))
        }
    };

    let genesis: GenesisFile = serde_json::from_str(genesis_str)?;

    tracing::info!("Parsed Genesis File and Beginning Hydration");

    let block = BlockActiveModel {
        era: Set(0),
        hash: Set(vec![]),
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        ..Default::default()
    };

    let block = block.insert(conn).await?;

    for data in genesis {
        let hash = hex::decode(data.hash)?;

        let transaction = TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(hash),
            is_valid: Set(true),
            payload: Set(vec![]),
            tx_index: Set(0),
            ..Default::default()
        };

        let transaction = transaction.insert(conn).await?;

        let payload = bs58::decode(data.address).into_vec()?;

        let address = AddressActiveModel {
            payload: Set(payload),
            ..Default::default()
        };

        let address = address.insert(conn).await?;

        let tx_output = TransactionOutputActiveModel {
            address_id: Set(address.id),
            tx_id: Set(transaction.id),
            payload: Set(vec![]),
            output_index: Set(0),
            ..Default::default()
        };

        tx_output.save(conn).await?;
    }

    Ok(())
}
