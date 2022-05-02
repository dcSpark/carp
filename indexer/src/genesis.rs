use std::fs;

use anyhow::anyhow;

use cardano_multiplatform_lib::{
    address::ByronAddress,
    genesis::byron::{
        config::GenesisData,
        parse::{parse, redeem_pubkey_to_txid},
    },
    utils::Value,
};
use entity::sea_orm::Iterable;
use entity::{
    prelude::{
        Address, AddressActiveModel, BlockActiveModel, Transaction, TransactionActiveModel,
        TransactionModel, TransactionOutput, TransactionOutputActiveModel,
    },
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set,
        TransactionTrait,
    },
};
use futures::future::try_join;
use migration::DbErr;

use crate::byron::blake2b256;

const GENESIS_MAINNET: &str = "./genesis/mainnet-byron-genesis.json";
const GENESIS_TESTNET: &str = "./genesis/testnet-byron-genesis.json";

pub async fn process_genesis(conn: &DatabaseConnection, network: &str) -> anyhow::Result<()> {
    let genesis_path = match network {
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

    let file = fs::File::open(genesis_path).expect("Failed to open genesis file");
    let genesis_file: Box<GenesisData> = Box::new(parse(file));

    tracing::info!(
        "Finished parsing genesis file after {:?}",
        time_counter.elapsed()
    );
    time_counter = std::time::Instant::now();

    tracing::info!("Inserting genesis data into database...");
    conn.transaction(|txn| Box::pin(insert_genesis(txn, genesis_file)))
        .await?;

    tracing::info!(
        "Finished inserting genesis data after {:?}",
        time_counter.elapsed()
    );

    Ok(())
}

pub async fn insert_genesis(
    txn: &DatabaseTransaction,
    genesis_file: Box<GenesisData>,
) -> Result<(), DbErr> {
    let genesis_hash = genesis_file.genesis_prev.to_bytes();
    tracing::info!(
        "Starting sync based on genesis hash {}",
        hex::encode(genesis_hash.clone())
    );

    // note: strictly speaking, the epoch, height, etc. isn't defined for the genesis block
    // since it comes before the first Epoch Boundary Block (EBB)
    let block = BlockActiveModel {
        era: Set(0),
        hash: Set(genesis_hash),
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        ..Default::default()
    };

    let block = block.insert(txn).await?;

    // note: avvm added before non-avvm
    // https://github.com/input-output-hk/cardano-ledger/blob/ac51494e151af0ad99b937a787458ce71db0aaea/eras/byron/ledger/impl/src/Cardano/Chain/UTxO/GenesisUTxO.hs#L21

    let mut transactions: Vec<TransactionActiveModel> = vec![];
    // note: genesis file is a JSON structure, so there shouldn't be duplicate addresses
    // even across avvm and non-avvm it should be unique, otherwise two txs with the same tx hash would exist
    let mut addresses: Vec<AddressActiveModel> = vec![];
    let mut outputs: Vec<cardano_multiplatform_lib::TransactionOutput> = vec![];

    for (pub_key, amount) in genesis_file.avvm_distr.iter() {
        let (tx_hash, extended_addr) =
            redeem_pubkey_to_txid(&pub_key, Some(genesis_file.protocol_magic));
        let byron_addr =
            ByronAddress::from_bytes(extended_addr.to_address().as_ref().to_vec()).unwrap();

        transactions.push(TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(tx_hash.to_bytes().to_vec()),
            is_valid: Set(true),
            payload: Set(byron_addr.to_bytes()),
            tx_index: Set(transactions.len() as i32),
            ..Default::default()
        });

        addresses.push(AddressActiveModel {
            payload: Set(byron_addr.to_bytes()),
            ..Default::default()
        });

        outputs.push(cardano_multiplatform_lib::TransactionOutput::new(
            &byron_addr.to_address(),
            &Value::new(amount),
        ));
    }

    // note: empty on mainnet
    for (addr, amount) in genesis_file.non_avvm_balances.iter() {
        let byron_addr = ByronAddress::from_bytes(addr.as_ref().to_vec()).unwrap();

        let tx_hash = blake2b256(&addr.as_ref());

        // println!("{}", amount.to_str());
        // println!("{}", byron_addr.to_base58());
        // println!("{}", hex::encode(tx_hash));

        transactions.push(TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(tx_hash.to_vec()),
            is_valid: Set(true),
            payload: Set(byron_addr.to_bytes()),
            tx_index: Set(transactions.len() as i32),
            ..Default::default()
        });

        addresses.push(AddressActiveModel {
            payload: Set(byron_addr.to_bytes()),
            ..Default::default()
        });

        outputs.push(cardano_multiplatform_lib::TransactionOutput::new(
            &byron_addr.to_address(),
            &Value::new(amount),
        ));
    }

    let (inserted_txs, inserted_addresses) = try_join(
        bulk_insert_txs(txn, &transactions),
        Address::insert_many(addresses).exec_many_with_returning(txn),
    )
    .await?;

    let outputs_to_add =
        inserted_txs
            .iter()
            .zip(inserted_addresses)
            .enumerate()
            .map(|(i, (tx, addr))| TransactionOutputActiveModel {
                address_id: Set(addr.id),
                tx_id: Set(tx.id),
                payload: Set(outputs[i].to_bytes()),
                // recall: genesis txs are hashes of addresses
                // so all txs have a single output
                output_index: Set(0),
                ..Default::default()
            });
    TransactionOutput::insert_many(outputs_to_add)
        .exec_many_with_returning(txn)
        .await?;

    Ok(())
}

// https://github.com/SeaQL/sea-orm/issues/691
async fn bulk_insert_txs(
    txn: &DatabaseTransaction,
    transactions: &Vec<TransactionActiveModel>,
) -> Result<Vec<TransactionModel>, DbErr> {
    let mut result: Vec<TransactionModel> = vec![];
    for chunk in transactions
        .chunks((u16::MAX / <Transaction as EntityTrait>::Column::iter().count() as u16) as usize)
    {
        result.extend(
            Transaction::insert_many(chunk.to_vec())
                .exec_many_with_returning(txn)
                .await?,
        );
    }
    Ok(result)
}
