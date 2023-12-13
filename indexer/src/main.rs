use crate::sink::Sink;
use crate::sinks::CardanoSink;
use crate::sources::{CardanoSource, OuraSource};
use crate::types::StoppableService;
use anyhow::{anyhow, Context};
use clap::Parser;
use dcspark_blockchain_source::{GetNextFrom, Source};
use migration::async_std::path::PathBuf;
use oura::sources::BearerKind;
use serde::Deserialize;
use std::borrow::Cow;
use std::fs::File;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
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
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum DbConfig {
    Postgres { database_url: String },
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
    Oura { socket: String, bearer: BearerKind },
    CardanoNet { relay: (Cow<'static, str>, u16) },
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
        .with_default(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    let running = Arc::new(AtomicBool::new(true));
    let processing_finished = Arc::new(AtomicBool::new(false));

    let r = running.clone();
    let p = processing_finished.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        while !p.load(Ordering::SeqCst) {}
        exit(0);
    })
    .expect("Error setting terminate handler");

    tracing::info!("{}", "Starting Carp");

    let Cli { plan, config_path } = Cli::parse();

    tracing::info!("Execution plan {}", plan);
    let exec_plan = Arc::new(ExecutionPlan::load_from_file(&plan)?);

    let config = if let Some(config_path) = config_path {
        tracing::info!("Config file {:?}", config_path);
        let file = File::open(&config_path).with_context(|| {
            format!(
                "Cannot open config file {path}",
                path = config_path.display()
            )
        })?;
        let config: Config = serde_yaml::from_reader(file).with_context(|| {
            format!(
                "Cannot parse config file {path}",
                path = config_path.display()
            )
        })?;
        config
    } else {
        dotenv::dotenv().ok();

        let carp_config = std::env::var("CARP_CONFIG")
            .expect("env CARP_CONFIG not found and --config-path not specified");
        let config: Config = serde_json::from_str(&carp_config)
            .with_context(|| format!("Cannot parse config string {carp_config}"))?;

        config
    };

    let (network, mut sink) = match config.sink {
        SinkConfig::Cardano { ref network, .. } => (
            network.clone(),
            CardanoSink::new(config.sink, exec_plan)
                .await
                .context("Can't create cardano sink")?,
        ),
    };

    tracing::info!("Passed start_block config: {:?}", config.start_block);

    let start_from_candidates = match config.start_block {
        None => sink
            .get_latest_points(15)
            .await
            .context("Can't get starting point from sink"),
        Some(block_hash) => sink
            .start_from(Option::from(block_hash))
            .await
            .context("Can't get starting point from config")
    }?;

    let start_from = start_from_candidates
        .last()
        .cloned()
        .ok_or_else(|| anyhow!("Starting points list is empty"))?;

    tracing::info!("Using start_from: {:?}", start_from);

    match &config.source {
        SourceConfig::Oura { .. } => {
            let source = OuraSource::new(config.source, network, start_from_candidates)
                .context("Can't create oura source")?;

            main_loop(source, sink, start_from, running, processing_finished).await
        }
        SourceConfig::CardanoNet { relay } => {
            let base_config = match network.as_ref() {
                "mainnet" => dcspark_blockchain_source::cardano::NetworkConfiguration::mainnet(),
                "preprod" => dcspark_blockchain_source::cardano::NetworkConfiguration::preprod(),
                "preview" => dcspark_blockchain_source::cardano::NetworkConfiguration::preview(),
                _ => return Err(anyhow::anyhow!("network not supported by source")),
            };

            let network_config = dcspark_blockchain_source::cardano::NetworkConfiguration {
                relay: relay.clone(),
                from: start_from.clone(),
                ..base_config
            };

            let source = CardanoSource::new(network_config).await?;

            main_loop(source, sink, start_from, running, processing_finished).await
        }
    };

    Ok(())
}

async fn main_loop<S>(
    source: S,
    sink: CardanoSink,
    start_from: <S as Source>::From,
    running: Arc<AtomicBool>,
    processing_finished: Arc<AtomicBool>,
) where
    S: Source<From = <CardanoSink as Sink>::From, Event = <CardanoSink as Sink>::Event>
        + StoppableService
        + Send,
    <S as Source>::Event: GetNextFrom,
    <S as Source>::From: Clone,
{
    let mut engine = engine::FetchEngine::new(source, sink, running);

    if let Err(error) = engine.fetch_and_process(start_from).await {
        tracing::error!(%error, "Processing loop finished with error, stopping engine");
    } else {
        tracing::info!("Processing loop finished successfully, stopping engine");
    }
    if let Err(error) = engine.stop().await {
        tracing::error!(%error, "Couldn't stop engine successfully");
    } else {
        tracing::info!("Engine is stopped successfully");
    }
    processing_finished.store(true, Ordering::SeqCst);
}
