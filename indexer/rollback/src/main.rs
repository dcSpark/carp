use clap::{Parser, Subcommand};
use dotenv::dotenv;
use entity::sea_orm::Database;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait},
};
use tracing_subscriber::prelude::*;

#[derive(Parser)]
#[clap(name = "subcommand")]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Will discard any block AFTER this block height (note: NOT slot)
    Height {
        height: i64,
    },
    // Will discard any block AFTER this epoch
    Epoch {
        epoch: i64,
    },
    // Will discard any block AFTER this era
    Era {
        era: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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

    dotenv().ok();

    let postgres_url = std::env::var("DATABASE_URL").expect("env DATABASE_URL not found");

    tracing::info!("{}", "Connecting to database...");
    let conn = Database::connect(&postgres_url).await?;

    tracing::info!(
        "{}",
        "Starting rollback. Note: rollbacks are not very fast. Expect a few minutes per epoch"
    );
    let rollback_start = std::time::Instant::now();
    match &args.action {
        Action::Height { height } => {
            tracing::info!("Rolling back to height {}", height);
            Block::delete_many()
                .filter(BlockColumn::Height.gt(*height))
                .exec(&conn)
                .await?;
        }
        Action::Epoch { epoch } => {
            tracing::info!("Rolling back to epoch {}", epoch);
            Block::delete_many()
                .filter(BlockColumn::Epoch.gt(*epoch))
                .exec(&conn)
                .await?;
        }
        Action::Era { era } => {
            tracing::info!("Rolling back to era {}", era);
            Block::delete_many()
                .filter(BlockColumn::Era.gt(*era))
                .exec(&conn)
                .await?;
        }
    }

    let time_taken = rollback_start.elapsed();
    tracing::info!("Rollback completed after {:?}", time_taken);

    Ok(())
}
