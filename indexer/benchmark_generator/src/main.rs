use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use std::borrow::Cow;
use std::fs::File;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::prelude::*;
use entity::sea_orm::Database;
use entity::sea_orm::QueryFilter;

use entity::{
    prelude::*,
    block::*,
    transaction::*,
    sea_orm::{
        entity::*, prelude::*, ColumnTrait, Condition, DatabaseTransaction, QueryOrder, Set,TransactionTrait, DatabaseConnection, EntityTrait, QuerySelect
    },
};

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

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Cli {
    /// path to config file
    #[clap(long, value_parser)]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let result = _main().await;
    result.unwrap();
}

async fn _main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();

    let Cli { config_path } = Cli::parse();

    tracing::info!("Config file {:?}", config_path);
    let file = File::open(&config_path).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;
    let config: DbConfig = serde_yaml::from_reader(file).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;
    let (
        user, password, host, port, db
    ) = match config {
        DbConfig::Postgres { host, port, user, password, db } => {
            (user, password, host, port, db)
        }
    };

    let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");
    tracing::info!("Connection url {:?}", url);
    let conn = Database::connect(&url).await?;
    tracing::info!("Connection success");
    let mut transactions = Transaction::find().order_by_asc(TransactionColumn::Id).paginate(&conn, 256);
    tracing::info!("Total transactions: {:?}", transactions.num_items().await.unwrap());
    tracing::info!("Total pages: {:?}", transactions.num_pages().await.unwrap());
    Ok(())
}
