use std::sync::Arc;

use anyhow::anyhow;
use dotenv::dotenv;

use entity::sea_orm::Database;
use oura::sources::IntersectArg;
use tasks::execution_plan::ExecutionPlan;
use tracing_subscriber::prelude::*;

mod genesis;
mod perf_aggregator;
mod postgres_sink;
mod setup;
mod types;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Path of the execution plan to use
    #[clap(short, long, default_value = "execution_plans/default.toml")]
    plan: String,

    /// Starting block hash. This will NOT rollback the database (use the rollback util for that)
    /// This is instead meant to make it easier to write database migrations
    #[clap(short, long)]
    start_block: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("oura", tracing::Level::WARN)
        .with_target("carp", tracing::Level::TRACE)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    tracing::info!("{}", "Starting Carp");

    dotenv().ok();
    let args = Args::parse();

    let network = std::env::var("NETWORK").expect("env NETWORK not found");
    let socket = std::env::var("SOCKET").expect("env SOCKET not found");

    let postgres_url = std::env::var("DATABASE_URL").expect("env DATABASE_URL not found");

    tracing::info!("Execution plan {}", args.plan);
    let exec_plan = Arc::new(ExecutionPlan::load_execution_plan(&args.plan));

    tracing::info!("{}", "Connecting to database...");
    let conn = Database::connect(&postgres_url).await?;

    tracing::info!("{}", "Getting the latest block synced from DB");

    // For rollbacks
    let points = &match &args.start_block {
        None => setup::get_latest_points(&conn).await?,
        Some(block) => setup::get_specific_point(&conn, block).await?,
    };
    let intersect = match points {
        points if points.is_empty() => {
            // insert genesis then fetch points again
            genesis::process_genesis(&conn, &network, exec_plan.clone()).await?;
            // we need a special intersection type when bootstrapping from genesis
            IntersectArg::Origin
        }
        points => {
            let last_point = points.last().unwrap();
            tracing::info!(
                "Starting sync at block #{} ({})",
                last_point.0,
                last_point.1
            );
            // if last block sync'd was at slot 0,
            // that means it was the genesis block so we start from origin
            match last_point.0 {
                0 => IntersectArg::Origin,
                _ => IntersectArg::Fallbacks(points.clone()),
            }
        }
    };

    let (handles, input) = setup::oura_bootstrap(intersect, &network, socket)?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    let initial_point = args.start_block.as_ref().map(|_| points.first().unwrap());
    sink_setup.start(input, exec_plan, initial_point).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
