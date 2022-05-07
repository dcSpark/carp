use anyhow::anyhow;
use dotenv::dotenv;

use entity::sea_orm::Database;
use oura::sources::IntersectArg;
use tracing_subscriber::prelude::*;

mod byron;
mod era_common;
mod genesis;
mod multiera;
mod perf_aggregator;
mod postgres_sink;
mod relation_map;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::INFO)
        .with_target("oura", tracing::Level::INFO)
        .with_target("oura_postgres_sink", tracing::Level::INFO);
    // .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    tracing::info!("{}", "Starting oura-postgres-sink");

    dotenv().ok();

    let network = std::env::var("NETWORK").expect("env NETWORK not found");
    let socket = std::env::var("SOCKET").expect("env SOCKET not found");

    let postgres_url = std::env::var("DATABASE_URL").expect("env DATABASE_URL not found");

    tracing::info!("{}", "Connecting to database...");
    let conn = Database::connect(&postgres_url).await?;

    tracing::info!("{}", "Getting the latest block synced from DB");

    // For rollbacks
    let intersect = match &setup::get_latest_points(&conn).await? {
        points if points.is_empty() => {
            // insert genesis then fetch points again
            genesis::process_genesis(&conn, &network).await?;
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

    sink_setup.bootstrap(input).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
