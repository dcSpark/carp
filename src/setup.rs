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
    genesis_hash: &str,
    network: &str,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    // we need a special intersection type when bootstrapping from genesis
    let intersect = match points.last() {
        None => Err(anyhow!("Missing intersection point for bootstrapping")),
        Some(point) if point.1 == genesis_hash => Ok(IntersectArg::Origin),
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

    tracing::info!("{}", "Attempting to connect to node...");

    let (source_handle, source_rx) = source_setup.bootstrap().map_err(|e| {
        tracing::error!("{}", e);
        anyhow!("failed to bootstrap source")
    })?;

    tracing::info!("{}", "Connection to node established");

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap filter"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}

const GENESIS_MAINNET: &str = include_str!("../genesis/mainnet.json");
const GENESIS_TESTNET: &str = include_str!("../genesis/testnet.json");

pub fn get_genesis_hash(network: &str) -> anyhow::Result<&str> {
    // TODO: avoid hard-coding these and instead pull from genesis file
    // https://github.com/dcSpark/oura-postgres-sink/issues/8
    match network {
        "mainnet" => Ok("5f20df933584822601f9e3f8c024eb5eb252fe8cefb24d1317dc3d432e940ebb"),
        "testnet" => Ok("96fceff972c2c06bd3bb5243c39215333be6d56aaf4823073dca31afe5038471"),
        rest => Err(anyhow!(
            "{} is invalid. NETWORK must be either mainnet or testnet",
            rest
        ))
    }
}
pub async fn insert_genesis(conn: &DatabaseConnection, genesis_hash: &str, network: &str) -> anyhow::Result<()> {
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
        hash: Set(hex::decode(genesis_hash)?),
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        ..Default::default()
    };

    let block = block.insert(conn).await?;

    for data in genesis {
        let tx_hash = hex::decode(data.hash)?;

        let transaction = TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(tx_hash),
            is_valid: Set(true),
            // TODO: payload
            payload: Set(vec![]),
            // TODO: index
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
            // TODO: payload
            payload: Set(vec![]),
            output_index: Set(data.index.try_into().unwrap()),
            ..Default::default()
        };

        tx_output.save(conn).await?;
    }

    Ok(())
}
