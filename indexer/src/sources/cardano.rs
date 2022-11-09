use crate::{common::CardanoEventType, types::StoppableService};
use anyhow::Context as _;
use async_trait::async_trait;
use dcspark_blockchain_source::{
    cardano::{
        BlockEvent, CardanoNetworkEvent, CardanoSource as WrappedCardanoSource,
        NetworkConfiguration, Point, Tip,
    },
    multiverse::rollback::{Event as RollbackOrEvent, ForkHandlingSource},
    Source,
};
use dcspark_core::BlockId;
use multiverse::Multiverse;
use std::time::Duration;

type CardanoSourceEvent = CardanoNetworkEvent<BlockEvent, Tip>;

pub struct CardanoSource {
    wrapped_source: ForkHandlingSource<
        BlockId,
        CardanoSourceEvent,
        WrappedCardanoSource,
        RollbackOrEvent<CardanoSourceEvent, Point>,
    >,
    configuration: NetworkConfiguration,
}

#[async_trait]
impl StoppableService for CardanoSource {
    async fn stop(self) -> anyhow::Result<()> {
        self.wrapped_source.into_inner().stop().await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Source for CardanoSource {
    type Event = crate::common::CardanoEventType;

    type From = Point;

    #[tracing::instrument(skip(self))]
    async fn pull(&mut self, from: &Self::From) -> anyhow::Result<Option<Self::Event>> {
        let maybe_event = self.wrapped_source.pull(&Some(from.clone())).await?;

        if let Some(event) = maybe_event {
            match event {
                RollbackOrEvent::Rollback(point) => {
                    let event = match point {
                        Point::Origin => unreachable!(),
                        Point::BlockHeader { slot_nb, hash } => CardanoEventType::RollBack {
                            block_slot: slot_nb.into(),
                            block_hash: hash.to_string(),
                        },
                    };

                    Ok(Some(event))
                }
                RollbackOrEvent::InnerEvent(CardanoNetworkEvent::Tip(_)) => Ok(None),
                RollbackOrEvent::InnerEvent(CardanoNetworkEvent::Block(block_event)) => {
                    if !block_event.is_boundary_block {
                        tracing::debug!(id = %block_event.id, "block event received");
                        Ok(Some(CardanoEventType::Block {
                            cbor_hex: hex::encode(block_event.raw_block.as_ref()),
                            epoch: block_event.epoch.or_else(|| {
                                self.configuration
                                    .shelley_era_config
                                    .absolute_slot_to_epoch(block_event.slot_number.into())
                            }),
                            epoch_slot: Some(block_event.slot_number.into()),
                            block_number: block_event.block_number.into(),
                            block_hash: block_event.id.to_string(),
                            block_slot: block_event.slot_number.into(),
                        }))
                    } else {
                        // boundary blocks make the byron task crash, so we skipped them by pulling
                        // again
                        // we shouldn't return None here, since it will enter an infinite loop
                        self.pull(&Point::BlockHeader {
                            slot_nb: block_event.slot_number,
                            hash: block_event.id,
                        })
                        .await
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl CardanoSource {
    pub async fn new(configuration: NetworkConfiguration) -> anyhow::Result<Self> {
        WrappedCardanoSource::connect(&configuration, Duration::from_millis(5000))
            .await
            .and_then(|cardano_source| {
                Multiverse::temporary()
                    .context("failed to create temporary multiverse")
                    .map(|multiverse| ForkHandlingSource::new(multiverse, 10, cardano_source))
            })
            .map(|wrapped_source| Self {
                configuration,
                wrapped_source,
            })
    }
}
