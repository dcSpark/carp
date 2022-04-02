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

    let network = std::env::var("NETWORK").expect("env NETWORK not found");
    let socket = std::env::var("SOCKET").expect("env SOCKET not found");

    let postgres_host = std::env::var("POSTGRES_HOST").expect("env POSTGRES_HOST not found");
    let postgres_port = std::env::var("POSTGRES_PORT").expect("env POSTGRES_PORT not found");
    let postgres_db = std::env::var("POSTGRES_DB").expect("env POSTGRES_DB not found");

    let postgres_user_file = std::env::var("POSTGRES_USER_FILE").expect("env POSTGRES_USER_FILE not found");
    let postgres_password_file = std::env::var("POSTGRES_PASSWORD_FILE").expect("env POSTGRES_PASSWORD_FILE not found");

    let postgres_user = fs::read_to_string(postgres_user_file)
        .expect("Cannot read POSTGRES_USER_FILE");
    let postgres_password = fs::read_to_string(postgres_password_file)
        .expect("Cannot read POSTGRES_PASSWORD_FILE");

    let url = format!("postgresql://{postgres_user}:{postgres_password}@{postgres_host}:{postgres_port}/{postgres_db}");

    let conn = Database::connect(&url).await?;

    // For rollbacks
    let points = match setup::get_latest_points(&conn).await? { 
        points if points.len() == 0 => {
            // insert genesis then fetch points again
            setup::insert_genesis(&conn, &network).await?;
            setup::get_latest_points(&conn).await?
        },
        points => points,
    };

    let (handles, input) = setup::oura_bootstrap(points, &network, socket)?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    sink_setup.bootstrap(input).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
