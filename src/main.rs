use std::{str::FromStr, sync::Arc};

use anyhow::anyhow;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    model::EventData,
    pipelining::{FilterProvider, SourceProvider},
    sources::{n2n, AddressArg, BearerKind, MagicArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use entity::{
    prelude::{Block, BlockActiveModel, BlockColumn, BlockModel},
    sea_orm::{prelude::*, Database, QueryOrder, QuerySelect, Set, TransactionTrait},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: use an environment variable before going to production
    let conn = Database::connect("postgresql://root:root@localhost:5432/cardano").await?;

    // For rollbacks
    let blocks: Vec<BlockModel> = Block::find()
        .order_by_desc(BlockColumn::Id)
        .limit(2160)
        .all(&conn)
        .await?;

    let magic = MagicArg::from_str("testnet").map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known, None));

    let mapper = mapper::Config {
        include_block_end_events: true,
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2n::Config {
        address: AddressArg(
            BearerKind::Tcp,
            "relays-new.cardano-testnet.iohkdev.io:3001".to_string(),
        ),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![
        String::from("Transaction"),
        String::from("BlockEnd"),
        String::from("Rollback"),
    ]);

    let filter_setup = selection::Config { check };

    let (source_handle, source_rx) = source_setup
        .bootstrap()
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    let sink_handle = tokio::spawn(async move {
        loop {
            let event = filter_rx.recv()?;

            let data = event.data.clone();

            match data {
                EventData::Block(block_record) => {
                    let hash = hex::decode(&block_record.hash)?;
                    let payload = hex::decode(block_record.cbor_hex.as_ref().unwrap())?;

                    conn.transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                            let block = BlockActiveModel {
                                era: Set(0),
                                hash: Set(hash),
                                height: Set(block_record.number as i32),
                                epoch: Set(0),
                                slot: Set(block_record.slot as i32),
                                payload: Set(payload),
                                ..Default::default()
                            };

                            block.save(txn).await?;

                            Ok(())
                        })
                    })
                    .await?;

                    println!(
                        "inserted block    - slot: {}, hash: {}",
                        block_record.slot, block_record.hash
                    );
                }
                EventData::RollBack {
                    block_hash,
                    block_slot,
                } => {
                    Block::delete_many()
                        .filter(BlockColumn::Slot.gte(block_slot))
                        .exec(&conn)
                        .await?;

                    println!(
                        "rollback complete - slot: {}, hash: {}",
                        block_slot, block_hash
                    );
                }
                _ => (),
            }
        }

        #[allow(unreachable_code)]
        Ok::<(), _>(())
    });

    source_handle.join().map_err(|_| anyhow!(""))?;
    filter_handle.join().map_err(|_| anyhow!(""))?;
    sink_handle.await.map_err(|_| anyhow!(""))?
}
