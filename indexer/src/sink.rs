use crate::perf_aggregator::PerfAggregator;
use async_trait::async_trait;
use dcspark_blockchain_source::{EventObject, PullFrom};
use entity::block::EraValue;

#[async_trait]
pub trait Sink {
    type From: PullFrom + Clone;
    type Event: EventObject;

    async fn start_from(&mut self, from: Option<String>) -> anyhow::Result<Vec<Self::From>>;
    async fn process(
        &mut self,
        event: Self::Event,
        perf_aggregator: &mut PerfAggregator,
        latest_era: &mut Option<EraValue>,
    ) -> anyhow::Result<()>;
}
