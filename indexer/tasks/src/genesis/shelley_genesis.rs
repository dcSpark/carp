use std::io::Cursor;
use std::path::PathBuf;

use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::task_macro::*;
use cml_chain::genesis::shelley::config::ShelleyGenesisData;
use cml_chain::transaction::AlonzoFormatTxOut;
use cml_chain::Serialize as _;
use cml_core::serialization::ToBytes;
use cml_crypto::{blake2b256, RawBytesEncoding};
use entity::block::{self, EraValue};
use entity::stake_credential;
use entity::{
    prelude::*,
    sea_orm::{entity::*, prelude::*, Condition, DatabaseTransaction},
};
use hex::ToHex;
use sea_orm::{QueryOrder, QuerySelect as _};
use tokio::io::AsyncReadExt as _;

carp_task! {
  name ShelleyGenesisBlockTask;
  configuration EmptyConfig;
  doc "Adds the block to the database";
  era shelley_genesis;
  dependencies [];
  read [];
  write [genesis_block];
  should_add_task |_block, _properties| {
    true
  };
  execute |_previous_data, task| handle_block(
      task.db_tx,
      task.block
  );
  merge_result |_previous_data, _result| {
  };
}

async fn handle_block(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, PathBuf, BlockGlobalInfo>,
) -> Result<(), DbErr> {
    if Genesis::find()
        .filter(GenesisColumn::Era.eq(i32::from(EraValue::Shelley)))
        .limit(1)
        .one(db_tx)
        .await?
        .is_some()
    {
        // There are two cases where we need to run the code in this task
        // 1. We have a new block with the Shelley era.
        // 2. We skipped the Shelley era and we got a block with a newer era.
        //
        // However, if we get a new era directly (like Conway), we need to know
        // if we've seen Shelley before or not. That's the reason we need this
        // condition.
        //
        // Note: Remember that the genesis file for each era is different.
        return Ok(());
    }

    let mut buffer = Vec::new();

    tokio::fs::File::open(block.1)
        .await
        .unwrap()
        .read_to_end(&mut buffer)
        .await
        .unwrap();

    let genesis = cml_chain::genesis::shelley::parse::parse_genesis_data(Cursor::new(buffer))
        .expect("Failed to parse genesis");

    let (latest_block_height, latest_block_epoch) = Block::find()
        .order_by_desc(block::Column::Height)
        .limit(1)
        .one(db_tx)
        .await?
        .map(|block| (block.height, block.epoch))
        .unwrap_or_default();

    // assuming that hard forks can only happen at epoch boundaries?
    let start_epoch = latest_block_epoch + 1;

    // TODO: these values should come from the byron genesis, but for the
    // existing networks the shelley and byron epoch lenghts are the same, and
    // for all of them the slot duration is 20 seconds too.
    // potentially we may want to add an entry in the era table with values for these though?
    // or we could read the genesis file here.
    let byron_slot_duration = 20;
    let epoch_length_in_byron_slots = genesis.epoch_length / byron_slot_duration;

    let first_slot = (block.2.epoch_slot.unwrap() / epoch_length_in_byron_slots
        * epoch_length_in_byron_slots) as i64;

    let inserted_block = Block::insert(BlockActiveModel {
        era: Set(EraValue::Shelley.into()),
        height: Set(latest_block_height + 1),
        epoch: Set(start_epoch),
        payload: Set(None),
        tx_count: Set(genesis.initial_funds.len().try_into().unwrap()),
        // TODO: what should we hash?
        hash: Set(b"shelley-genesis".to_vec()),
        slot: Set(first_slot.try_into().unwrap()),
        ..Default::default()
    })
    .exec_with_returning(true, db_tx)
    .await?
    .unwrap();

    Genesis::insert(GenesisActiveModel {
        era: Set(i32::from(EraValue::Shelley)),
        block_id: Set(inserted_block.id),
        block_height: Set(latest_block_height + 1),
        first_slot: Set(first_slot),
        start_epoch: Set(start_epoch.into()),
        epoch_length_seconds: Set(genesis.epoch_length as i64),
    })
    .exec(db_tx)
    .await?;

    let stake_credentials =
        handle_initial_funds((block.0, &genesis, block.2), inserted_block, db_tx).await?;

    if let Some(staking) = genesis
        .staking
        .as_ref()
        .filter(|staking| !staking.stake.is_empty())
    {
        entity::stake_delegation::Entity::insert_many(staking.stake.iter().map(
            |(stake_credential, pool)| {
                let stake_credential_entry = stake_credentials
                    .iter()
                    .find(|inserted_credential| {
                        inserted_credential.credential
                            == cml_chain::certs::StakeCredential::new_pub_key(*stake_credential)
                                .to_cbor_bytes()
                    })
                    .unwrap();

                entity::stake_delegation::ActiveModel {
                    pool_credential: Set(Some(pool.to_raw_bytes().to_vec())),
                    previous_pool: Set(None),
                    stake_credential: Set(stake_credential_entry.id),
                    // Note: this is not really the tx of the delegation, but the tx
                    // of the utxo with the initial funds. However there is no other
                    // tx to assign this to otherwise.
                    tx_id: Set(stake_credential_entry.first_tx),
                    ..Default::default()
                }
            },
        ))
        .exec(db_tx)
        .await?;
    }

    Ok(())
}

async fn handle_initial_funds(
    block: (&str, &ShelleyGenesisData, &BlockGlobalInfo),
    inserted_block: BlockModel,
    db_tx: &DatabaseTransaction,
) -> Result<Vec<StakeCredentialModel>, DbErr> {
    if block.1.initial_funds.is_empty() {
        return Ok(vec![]);
    }

    let inserted_transactions =
        Transaction::insert_many(block.1.initial_funds.keys().map(|address| {
            let tx_id = blake2b256(&address.to_raw_bytes());

            TransactionActiveModel {
                block_id: Set(inserted_block.id),
                hash: Set(tx_id.to_vec()),
                is_valid: Set(true),
                payload: Set(vec![]),
                tx_index: Set(0),
                ..Default::default()
            }
        }))
        .exec_many_with_returning(db_tx)
        .await?;

    let inserted_addresses = Address::insert_many(
        block
            .1
            .initial_funds
            .iter()
            .zip(inserted_transactions.iter())
            .map(|((address, _), inserted_tx)| AddressActiveModel {
                payload: Set(address.to_raw_bytes()),
                first_tx: Set(inserted_tx.id),
                ..Default::default()
            }),
    )
    .exec_many_with_returning(db_tx)
    .await?;

    TransactionOutput::insert_many(block.1.initial_funds.iter().zip(inserted_addresses).map(
        |((address, coin), address_model)| {
            TransactionOutputActiveModel {
                address_id: Set(address_model.id),
                tx_id: Set(address_model.first_tx),
                payload: Set(
                    cml_chain::transaction::TransactionOutput::AlonzoFormatTxOut(
                        AlonzoFormatTxOut::new(
                            address.clone(),
                            cml_chain::Value::new(*coin, Default::default()),
                        ),
                    )
                    .to_cbor_bytes(),
                ),
                output_index: Set(0),
                ..Default::default()
            }
        },
    ))
    .exec_many_with_returning(db_tx)
    .await?;

    let inserted_credentials = StakeCredential::insert_many(
        block
            .1
            .initial_funds
            .iter()
            .zip(inserted_transactions)
            .filter_map(|((address, _), inserted_tx)| {
                let stake_credential = match address {
                    cml_chain::address::Address::Base(base) => Some(base.stake.clone()),
                    // TODO: this doesn't seem possible?
                    cml_chain::address::Address::Ptr(_) => todo!(),
                    cml_chain::address::Address::Enterprise(_) => None,
                    cml_chain::address::Address::Reward(_) => None,
                    cml_chain::address::Address::Byron(_) => None,
                };

                if let Some(stake_credential) = stake_credential {
                    Some(StakeCredentialActiveModel {
                        credential: Set(stake_credential.to_cbor_bytes()),
                        first_tx: Set(inserted_tx.id),
                        ..Default::default()
                    })
                } else {
                    None
                }
            }),
    )
    .exec_many_with_returning(db_tx)
    .await?;

    Ok(inserted_credentials)
}
