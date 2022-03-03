use anyhow::anyhow;
use dotenv::dotenv;

use entity::sea_orm::Database;

mod postgres_sink;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_test_writer().init();

    dotenv()?;

    let url = std::env::var("DATABASE_URL")?;
    let network = std::env::var("NETWORK")?;
    let socket = std::env::var("SOCKET")?;

    let conn = Database::connect(&url).await?;

    // For rollbacks
    let points = setup::get_latest_points(&conn).await?;

    if points.is_empty() {
        setup::insert_genesis(&conn, &network).await?;
    }

    let (handles, input) = setup::oura_bootstrap(points, &network, socket)?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    sink_setup.bootstrap(input).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
