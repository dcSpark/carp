extern crate core;

use crate::sink::Sink;
use crate::sinks::CardanoSink;
use crate::sources::OuraSource;
use crate::types::StoppableService;
use anyhow::Context;
use clap::Parser;
use dcspark_blockchain_source::{GetNextFrom, Source};
use migration::async_std::path::PathBuf;
use oura::sources::BearerKind;
use serde::Deserialize;
use std::borrow::Cow;
use std::fs::File;
use std::sync::Arc;
use tasks::execution_plan::ExecutionPlan;
use tracing_subscriber::prelude::*;

mod common;
mod engine;
mod genesis;
mod perf_aggregator;
mod sink;
mod sinks;
mod sources;
mod types;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Cli {
    /// Path of the execution plan to use
    #[clap(short, long, default_value = "execution_plans/default.toml")]
    plan: String,

    /// path to config file
    #[clap(long, value_parser)]
    config_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum DbConfig {
    Postgres {
        host: String,
        port: u64,
        user: String,
        password: String,
        db: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum SinkConfig {
    Cardano { db: DbConfig, network: String },
}

pub enum Network {}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum SourceConfig {
    Oura {
        network: String,
        socket: String,
        bearer: BearerKind,
    },
    DirectSource {},
    CardanoNet {
        relay: (Cow<'static, str>, u16),
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    source: SourceConfig,
    sink: SinkConfig,
    /// Starting block hash. This will NOT rollback the database (use the rollback util for that)
    /// This is instead meant to make it easier to write database migrations
    start_block: Option<String>,
}

#[allow(unreachable_patterns)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("oura", tracing::Level::WARN)
        .with_target("sled", tracing::Level::INFO)
        .with_target("carp", tracing::Level::TRACE)
        .with_target("cardano-net", tracing::Level::INFO)
        .with_target("cardano-sdk", tracing::Level::INFO)
        .with_target("dcspark-blockchain-source", tracing::Level::INFO)
        .with_default(tracing::Level::DEBUG);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    tracing::info!("{}", "Starting Carp");

    let Cli { plan, config_path } = Cli::parse();

    tracing::info!("Execution plan {}", plan);
    let exec_plan = Arc::new(ExecutionPlan::load_from_file(&plan));

    tracing::info!("Config file {:?}", config_path);
    let file = File::open(&config_path).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;
    let config: Config = serde_yaml::from_reader(file).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;

    let (network, mut sink) = match config.sink {
        SinkConfig::Cardano { ref network, .. } => (
            network.clone(),
            CardanoSink::new(config.sink, exec_plan).await?,
        ),
    };

    let start_from = sink.start_from(config.start_block).await?;

    match &config.source {
        SourceConfig::Oura { .. } => {
            let source = OuraSource::new(config.source, start_from.clone())?;
            let start_from = start_from.into_iter().next().unwrap();

            main_loop(source, sink, start_from).await
        }
        SourceConfig::CardanoNet { relay } => {
            let base_config = match network.as_ref() {
                "mainnet" => dcspark_blockchain_source::cardano::NetworkConfiguration::mainnet(),
                "preprod" => dcspark_blockchain_source::cardano::NetworkConfiguration::preprod(),
                "preview" => dcspark_blockchain_source::cardano::NetworkConfiguration::preview(),
                _ => return Err(anyhow::anyhow!("network not supported by source")),
            };

            // try to find a confirmed point.
            //
            // this way the multiverse can be temporary, which saves setting up the extra db
            // (at the expense of repulling some extra blocks at startup)
            let start_from = sink.get_latest_points(15).await?.last().unwrap().clone();

            let network_config = dcspark_blockchain_source::cardano::NetworkConfiguration {
                relay: relay.clone(),
                from: start_from.clone(),
                ..base_config
            };

            let source = sources::cardano::CardanoSource::new(network_config).await?;

            main_loop(source, sink, start_from).await
        }
        SourceConfig::DirectSource {} => todo!("not supported yet"),
    };

    Ok(())
}

async fn main_loop<S>(source: S, sink: CardanoSink, start_from: <S as Source>::From)
where
    S: Source<From = <CardanoSink as Sink>::From, Event = <CardanoSink as Sink>::Event>
        + StoppableService
        + Send,
    <S as Source>::Event: GetNextFrom,
    <S as Source>::From: Clone,
{
    let mut engine = engine::FetchEngine::new(source, sink);

    // TODO: add signal handling to stop the engine correctly
    //
    if let Err(error) = engine.fetch_and_process(start_from).await {
        tracing::error!(%error, "processing loop finished with error, stopping engine");
        if let Err(error) = engine.stop().await {
            tracing::error!(%error, "couldn't stop engine succesfully");
        }
    }
}
