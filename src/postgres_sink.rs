use oura::{model::EventData, pipelining::StageReceiver};

use entity::{
    prelude::{Block, BlockActiveModel, BlockColumn},
    sea_orm::{prelude::*, Set, TransactionTrait},
};
use migration::DbErr;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

impl<'a> Config<'a> {
    pub async fn bootstrap(&self, input: StageReceiver) -> anyhow::Result<()> {
        loop {
            let event = input.recv()?;

            let data = event.data.clone();

            match data {
                EventData::Block(block_record) => {
                    let hash = hex::decode(&block_record.hash)?;
                    let payload = hex::decode(block_record.cbor_hex.as_ref().unwrap())?;

                    self.conn
                        .transaction::<_, (), DbErr>(|txn| {
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
                        .exec(self.conn)
                        .await?;

                    println!(
                        "rollback complete - slot: {}, hash: {}",
                        block_slot, block_hash
                    );
                }
                _ => (),
            }
        }
    }
}
