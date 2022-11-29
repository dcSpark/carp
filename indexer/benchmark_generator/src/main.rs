use std::path::PathBuf;

use anyhow::{anyhow, Context};
use cardano_multiplatform_lib::address::{Address, StakeCredential};
use cardano_multiplatform_lib::crypto::TransactionHash;
use cardano_multiplatform_lib::ledger::common::value::{BigNum, Coin};
use cardano_multiplatform_lib::{TransactionInput, TransactionOutput};
use cardano_sdk::chain::TxHash;
use clap::Parser;
use dcspark_core::tx::{TransactionId, UtxoPointer};
use dcspark_core::{Balance, OutputIndex, Regulated, TokenId};
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
use std::collections::{HashMap, HashSet};
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
pub struct TxOutputIntent {
    address: Option<StakeCredential>,
    amount: cardano_multiplatform_lib::ledger::common::value::Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum TxEvent {
    // every from is shelley address. `to` can be any address
    // we store all amounts in this case: either shelley or byron outputs amounts, since
    // we can perform the selection for that
    FromParsed {
        to: Vec<TxOutputIntent>,
        fee: cardano_multiplatform_lib::ledger::common::value::Coin,
        // we can assume we can spend utxos with both credentials if we have multiple froms
        from: Vec<StakeCredential>,
    },
    // this applies when some of the from addresses are byron
    // we store shelley from and to intents
    // this way we can compute the balance change afterwards
    PartialParsed {
        to: Vec<TxOutputIntent>,
        // we store how much money we spent from the parsed addresses
        from: Vec<TxOutputIntent>,
    },
    Unparsed {
        tx: TransactionHash,
    },
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
    let mut shelley_first_tx = shelley_first_tx
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

    let mut previous_outputs = HashMap::<
        TransactionHash,
        HashMap<
            BigNum,
            (
                StakeCredential,
                cardano_multiplatform_lib::ledger::common::value::Value,
            ),
        >,
    >::new();

    while let Some(txs) = stream.try_next().await? {
        tracing::info!("handling page: {:?} of {:?}", current_page, total_pages);
        for tx in txs {
            let payload: &Vec<u8> = &tx.payload;
            match cardano_multiplatform_lib::Transaction::from_bytes(payload.clone()) {
                Ok(parsed) => {
                    let body = parsed.body();
                    // inputs handle
                    let inputs = body.inputs();

                    let mut is_from_partial = false;

                    // try to parse input addresses and put in the set
                    let mut input_addresses = HashSet::new();
                    let mut input_intents = Vec::new();
                    for input_index in 0..inputs.len() {
                        let input = inputs.get(input_index);
                        // try to find output and extract address from there
                        if let Some(outputs) =
                            &mut previous_outputs.get_mut(&input.transaction_id())
                        {
                            // we remove the spent input from the list
                            if let Some((cred, amount)) = outputs.remove(&input.index()) {
                                input_addresses.insert(cred.clone());
                                input_intents.push(TxOutputIntent {
                                    address: Some(cred),
                                    amount,
                                })
                            }
                        } else {
                            is_from_partial = true; // might be byron address or sth
                        }
                        // remove if whole transaction is spent
                        if previous_outputs
                            .get(&input.transaction_id())
                            .map(|outputs| outputs.is_empty())
                            .unwrap_or(false)
                        {
                            previous_outputs.remove(&input.transaction_id());
                        }
                    }

                    // outputs handle
                    let outputs = body.outputs();
                    let mut output_intents = vec![];
                    for output_index in 0..outputs.len() {
                        let output = outputs.get(output_index);
                        if let Some(credential) = output.address().payment_cred() {
                            let tx_outputs_previous = previous_outputs
                                .entry(
                                    TransactionHash::from_bytes(tx.hash.clone())
                                        .map_err(|err| anyhow!("err: {:?}", err))?,
                                )
                                .or_default();
                            tx_outputs_previous.insert(
                                BigNum::from(output_index as u64),
                                (credential.clone(), output.amount()),
                            );
                            output_intents.push(TxOutputIntent {
                                address: Some(credential),
                                amount: output.amount(),
                            })
                        } else {
                            // might be byron address
                            if !is_from_partial {
                                output_intents.push(TxOutputIntent {
                                    address: None,
                                    amount: output.amount(),
                                })
                            }
                        }
                    }
                    let event = if !is_from_partial {
                        TxEvent::FromParsed {
                            to: output_intents, // all output intents incl byron
                            fee: body.fee(),
                            from: Vec::from_iter(input_addresses.into_iter()), // input addresses are parsed fully
                        }
                    } else {
                        TxEvent::PartialParsed {
                            to: output_intents,  // can lack byron addresses
                            from: input_intents, // can lack byron addresses
                        }
                    };

                    out_file
                        .write_all(format!("{}\n", serde_json::to_string(&event)?).as_bytes())?;
                }
                Err(err) => {
                    let event = TxEvent::Unparsed {
                        tx: TransactionHash::from_bytes(tx.hash.clone())
                            .map_err(|err| anyhow!("err: {:?}", err))?,
                    };
                    out_file
                        .write_all(format!("{}\n", serde_json::to_string(&event)?).as_bytes())?;
                }
            }
        }
        current_page += 1;
    }
    Ok(())
}
