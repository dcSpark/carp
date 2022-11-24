use std::path::PathBuf;

use anyhow::{anyhow, Context};
use cardano_multiplatform_lib::ledger::common::value::Coin;
use cardano_multiplatform_lib::{TransactionInput, TransactionOutput};
use clap::Parser;
use entity::sea_orm::Database;
use entity::sea_orm::QueryFilter;
use entity::{
    block::*,
    prelude::*,
    sea_orm::{
        entity::*, prelude::*, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction,
        EntityTrait, QueryOrder, QuerySelect, Set, TransactionTrait,
    },
    transaction::*,
};
use futures::TryStreamExt;
use pallas::ledger::traverse::Era;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::prelude::*;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    db: DbConfig,
    tx_per_page: usize,
}

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Cli {
    /// path to config file
    #[clap(long, value_parser)]
    config_path: PathBuf,

    /// path to output file
    #[clap(long, value_parser)]
    output_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TxOutFormat {
    fee: Coin,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
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
        .with_target("sqlx", tracing::Level::WARN)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();

    let Cli {
        config_path,
        output_path,
    } = Cli::parse();

    tracing::info!("Config file {:?}", config_path);
    let file = File::open(&config_path).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;
    let config: Config = serde_yaml::from_reader(file).with_context(|| {
        format!(
            "Cannot read config file {path}",
            path = config_path.display()
        )
    })?;
    let (user, password, host, port, db) = match config.db {
        DbConfig::Postgres {
            host,
            port,
            user,
            password,
            db,
        } => (user, password, host, port, db),
    };

    let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");
    tracing::info!("Connection url {:?}", url);
    let conn = Database::connect(&url).await?;
    tracing::info!("Connection success");
    let shelley_first_blocks = Block::find()
        .filter(BlockColumn::Epoch.eq(208))
        .order_by_asc(BlockColumn::Id)
        .limit(256)
        .all(&conn)
        .await?;
    tracing::info!(
        "Shelley first blocks {:?}",
        shelley_first_blocks
            .iter()
            .map(|block| (block.id, block.height, block.epoch))
            .collect::<Vec<(i32, i32, i32)>>()
    );
    let shelley_first_blocks: Vec<i32> =
        shelley_first_blocks.iter().map(|block| block.id).collect();

    let mut condition = Condition::any();
    for block in shelley_first_blocks {
        condition = condition.add(TransactionColumn::BlockId.eq(block));
    }
    let shelley_first_tx: Vec<i64> = Transaction::find()
        .filter(condition)
        .order_by_asc(TransactionColumn::Id)
        .limit(1)
        .all(&conn)
        .await?
        .iter()
        .map(|tx| tx.id)
        .collect();
    let shelley_first_tx = shelley_first_tx
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("Can't find first tx"))?;
    tracing::info!("Shelley first tx, {:?}", shelley_first_tx);

    let mut transactions = Transaction::find()
        .filter(TransactionColumn::Id.gte(shelley_first_tx))
        .order_by_asc(TransactionColumn::Id)
        .paginate(&conn, config.tx_per_page);
    let total_transactions = transactions.num_items().await?;
    let total_pages = transactions.num_pages().await?;
    tracing::info!("Total transactions: {:?}", total_transactions);
    tracing::info!("Total pages: {:?}", total_pages);

    let mut current_page = transactions.cur_page();
    let mut stream = transactions.into_stream();
    let mut out_file = if output_path.exists() && output_path.is_file() {
        tracing::info!(
            "file {:?} already exists, adding lines to the end",
            output_path
        );
        File::open(output_path)
    } else {
        File::create(output_path)
    }?;

    while let Some(txs) = stream.try_next().await? {
        tracing::info!("handling page: {:?} of {:?}", current_page, total_pages);
        for tx in txs {
            let payload: &Vec<u8> = &tx.payload;
            match cardano_multiplatform_lib::Transaction::from_bytes(payload.clone()) {
                Ok(parsed) => {
                    let body = parsed.body();
                    let inputs = body.inputs();
                    let mut out_inputs = vec![];
                    for i in 0..inputs.len() {
                        out_inputs.push(inputs.get(i));
                    }

                    let outputs = body.outputs();
                    let mut out_outputs = vec![];
                    for i in 0..outputs.len() {
                        out_outputs.push(outputs.get(i));
                    }
                    let out = TxOutFormat {
                        fee: body.fee(),
                        inputs: out_inputs,
                        outputs: out_outputs,
                    };

                    out_file.write_all(format!("{}\n", serde_json::to_string(&out)?).as_bytes())?;
                }
                Err(err) => {
                    tracing::warn!("can't parse tx: error: {:?}, tx: {:?}", err, tx);
                }
            }
        }
        current_page += 1;
    }
    Ok(())
}
