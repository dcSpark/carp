extern crate core;

use std::fs::File;
use std::sync::Arc;

use anyhow::Context;

use tasks::execution_plan::ExecutionPlan;
use tracing_subscriber::prelude::*;

mod common;
mod engine;
mod genesis;
mod perf_aggregator;
mod sink;
mod sinks;
mod sources;
mod tracing_utils;
mod types;

use crate::common::CardanoEventType;
use crate::sink::Sink;
use crate::sinks::CardanoSink;
use crate::sources::OuraSource;
use crate::tracing_utils::setup_logging;
use clap::Parser;
use dcspark_blockchain_source::cardano::Point;
use dcspark_blockchain_source::Source;
use migration::async_std::path::PathBuf;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum SourceConfig {
    Oura { network: String, socket: String },
    DirectSource {},
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

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

    let mut sink: CardanoSink = match config.sink {
        SinkConfig::Cardano { .. } => CardanoSink::new(config.sink, exec_plan).await?,
        _ => todo!("not supported yet"),
    };

    let start_from = sink.start_from(config.start_block).await?;

    let mut engine = match &config.source {
        SourceConfig::Oura { .. } => {
            let source = OuraSource::new(config.source, start_from.clone())?;
            engine::FetchEngine::new(source, sink)
        }
        _ => todo!("not supported yet"),
    };

    engine
        .fetch_and_process(start_from.first().unwrap().clone())
        .await
}
