use anyhow::anyhow;
use dotenv::dotenv;
use std::fs;

use entity::sea_orm::Database;

mod postgres_sink;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt().with_test_writer().init();

    dotenv().ok();

    let network = std::env::var("NETWORK")?;
    let socket = std::env::var("SOCKET")?;

    let postgres_host = std::env::var("POSTGRES_HOST")?;
    let postgres_port = std::env::var("POSTGRES_PORT")?;
    let postgres_db = std::env::var("POSTGRES_DB")?;

    let postgres_user_file = std::env::var("POSTGRES_USER_FILE")?;
    let postgres_password_file = std::env::var("POSTGRES_PASSWORD_FILE")?;

    let postgres_user = fs::read_to_string(postgres_user_file)
        .expect("Cannot read POSTGRES_USER_FILE");
    let postgres_password = fs::read_to_string(postgres_password_file)
        .expect("Cannot read POSTGRES_PASSWORD_FILE");

    let url = format!("postgresql://{postgres_user}:{postgres_password}@{postgres_host}:{postgres_port}/{postgres_db}");

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
