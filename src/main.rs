use anyhow::anyhow;

use entity::sea_orm::Database;

mod postgres_sink;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_test_writer().init();

    // TODO: use an environment variable before going to production
    let conn = Database::connect("postgresql://root:root@localhost:5432/cardano").await?;

    // For rollbacks
    let points = setup::get_latest_points(&conn).await?;

    let (handles, input) = setup::oura_bootstrap(points)?;

    let sink_setup = postgres_sink::Config { conn: &conn };

    sink_setup.bootstrap(input).await?;

    for handle in handles {
        handle.join().map_err(|_| anyhow!(""))?;
    }

    Ok(())
}
