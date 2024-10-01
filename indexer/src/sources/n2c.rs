use async_trait::async_trait;
use dcspark_blockchain_source::{cardano::Point, Source};
use dcspark_core::StoppableService as _;

pub struct N2CSource(pub dcspark_blockchain_source::cardano::N2CSource);

#[async_trait]
impl crate::types::StoppableService for N2CSource {
    async fn stop(self) -> anyhow::Result<()> {
        self.0.stop().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Source for N2CSource {
    type Event = crate::common::CardanoEventType;

    type From = Point;

    #[tracing::instrument(skip(self))]
    async fn pull(&mut self, _from: &Self::From) -> anyhow::Result<Option<Self::Event>> {
        let event = self.0.pull(&()).await?;

        let Some(event) = event else {
            return Ok(None);
        };

        match event {
            dcspark_blockchain_source::cardano::N2CSourceEvent::RollBack {
                block_slot,
                block_hash,
            } => Ok(Some(crate::common::CardanoEventType::RollBack {
                block_slot,
                block_hash,
            })),
            dcspark_blockchain_source::cardano::N2CSourceEvent::Block(block_event) => {
                Ok(Some(crate::common::CardanoEventType::Block {
                    cbor_hex: hex::encode(block_event.raw_block),
                    epoch: block_event.epoch,
                    epoch_slot: Some(block_event.slot_number.into()),
                    block_number: block_event.block_number.into(),
                    block_hash: block_event.id.to_string(),
                    block_slot: block_event.slot_number.into(),
                }))
            }
        }
    }
}
