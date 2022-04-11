use anyhow::anyhow;
use dotenv::dotenv;

use entity::sea_orm::Database;
use tracing_subscriber::prelude::*;

mod postgres_sink;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("oura", tracing::Level::WARN)
        .with_target("oura_postgres_sink", tracing::Level::TRACE)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

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

    let genesis_hash = setup::get_genesis_hash(&network)?;
    // For rollbacks
    let points = match setup::get_latest_points(&conn).await? {
        points if points.is_empty() => {
            // insert genesis then fetch points again
            setup::insert_genesis(&conn, genesis_hash, &network).await?;
            setup::get_latest_points(&conn).await?
        }
        points => points,
    };

    match points.last() {
        None => Err(anyhow!("Missing intersection point for bootstrapping")),
        Some(point) => match point.0 {
            0 => {
                tracing::info!("Starting sync from genesis {}", genesis_hash);
                Ok(())
            }
            _ => {
                tracing::info!("Starting sync at block #{} ({})", point.0, point.1);
                Ok(())
            }
        },
    }?;

    let (handles, input) = setup::oura_bootstrap(points, genesis_hash, &network, socket)?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    sink_setup.bootstrap(input).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
