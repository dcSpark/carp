mod reparse;

use dotenv::dotenv;

use entity::sea_orm::Database;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::WARN)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    tracing::info!("{}", "Starting oura-postgres-sink reparse");

    dotenv().ok();

    let postgres_url = std::env::var("DATABASE_URL").expect("env DATABASE_URL not found");

    tracing::info!("{}", "Connecting to database...");
    let conn = Database::connect(&postgres_url).await?;
    reparse::start_reparse(conn).await?;

    Ok(())
}
