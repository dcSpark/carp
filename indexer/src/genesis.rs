use anyhow::anyhow;

use entity::{
    prelude::{
        AddressActiveModel, BlockActiveModel, TransactionActiveModel, TransactionOutputActiveModel,
    },
    sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionTrait},
};
use migration::DbErr;

use crate::types::GenesisFile;

const GENESIS_MAINNET: &str = include_str!("../genesis/mainnet.json");
const GENESIS_TESTNET: &str = include_str!("../genesis/testnet.json");

pub fn get_genesis_hash(network: &str) -> anyhow::Result<&str> {
    // TODO: avoid hard-coding these and instead pull from genesis file
    // https://github.com/dcSpark/oura-postgres-sink/issues/8
    match network {
        "mainnet" => Ok("5f20df933584822601f9e3f8c024eb5eb252fe8cefb24d1317dc3d432e940ebb"),
        "testnet" => Ok("96fceff972c2c06bd3bb5243c39215333be6d56aaf4823073dca31afe5038471"),
        rest => Err(anyhow!(
            "{} is invalid. NETWORK must be either mainnet or testnet",
            rest
        )),
    }
}
pub async fn process_genesis(
    conn: &DatabaseConnection,
    genesis_hash: &str,
    network: &str,
) -> anyhow::Result<()> {
    let genesis_str = match network {
        "mainnet" => GENESIS_MAINNET,
        "testnet" => GENESIS_TESTNET,
        rest => {
            return Err(anyhow!(
                "{} is invalid. NETWORK must be either mainnet or testnet",
                rest
            ))
        }
    };

    tracing::info!("Parsing genesis file...");

    let mut time_counter = std::time::Instant::now();
    let genesis_file: Box<GenesisFile> = Box::new(serde_json::from_str(genesis_str)?);
    tracing::info!(
        "Finished parsing genesis file after {:?}",
        time_counter.elapsed()
    );
    time_counter = std::time::Instant::now();

    tracing::info!("Inserting genesis data into database...");
    let genesis_hash: String = genesis_hash.to_owned();
    conn.transaction(|txn| Box::pin(insert_genesis(txn, genesis_file, genesis_hash)))
        .await?;

    tracing::info!(
        "Finished inserting genesis data after {:?}",
        time_counter.elapsed()
    );

    Ok(())
}

pub async fn insert_genesis(
    txn: &DatabaseTransaction,
    genesis_file: Box<GenesisFile>,
    genesis_hash: String,
) -> Result<(), DbErr> {
    let block = BlockActiveModel {
        era: Set(0),
        hash: Set(hex::decode(genesis_hash).unwrap()),
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        ..Default::default()
    };

    let block = block.insert(txn).await?;

    for data in genesis_file.iter() {
        let tx_hash = hex::decode(&data.hash).unwrap();

        let transaction = TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(tx_hash),
            is_valid: Set(true),
            // TODO: payload
            payload: Set(vec![]),
            // TODO: index
            tx_index: Set(0),
            ..Default::default()
        };

        let transaction = transaction.insert(txn).await?;

        let payload = bs58::decode(&data.address).into_vec().unwrap();

        let address = AddressActiveModel {
            payload: Set(payload),
            ..Default::default()
        };

        let address = address.insert(txn).await?;

        let tx_output = TransactionOutputActiveModel {
            address_id: Set(address.id),
            tx_id: Set(transaction.id),
            // TODO: payload
            payload: Set(vec![]),
            output_index: Set(data.index.try_into().unwrap()),
            ..Default::default()
        };

        tx_output.save(txn).await?;
    }

    Ok(())
}
