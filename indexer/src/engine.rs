use crate::common::GetNextFrom;
use crate::perf_aggregator::PerfAggregator;
use crate::sink::Sink;
use dcspark_blockchain_source::{PullFrom, Source};
use dcspark_core::BlockNumber;
use migration::async_std::sync::Mutex;
use oura::model::Event;
use std::sync::Arc;
use tasks::utils::TaskPerfAggregator;

pub struct FetchEngine<
    FromType: PullFrom + Clone,
    EventType,
    SourceType: Source<From = FromType, Event = EventType>,
    SinkType: Sink<From = FromType, Event = EventType>,
> {
    source: SourceType,
    sink: SinkType,
}

impl<
        FromType: PullFrom + Clone,
        EventType: GetNextFrom<From = FromType>,
        SourceType: Source<From = FromType, Event = EventType>,
        SinkType: Sink<From = FromType, Event = EventType>,
    > FetchEngine<FromType, EventType, SourceType, SinkType>
{
    pub fn new(
        source: SourceType,
        sink: SinkType,
    ) -> FetchEngine<FromType, EventType, SourceType, SinkType> {
        Self { source, sink }
    }

    pub async fn fetch_and_process(&mut self, from: FromType) -> anyhow::Result<()> {
        tracing::info!("{}", "Starting to process blocks");
        let mut pull_from = from;

        let mut perf_aggregator = PerfAggregator::new();

        loop {
            let event_fetch_start = std::time::Instant::now();
            let event = self.source.pull(&pull_from).await?;
            let event = if let Some(event) = event {
                event
            } else {
                continue;
            };
            perf_aggregator.block_fetch += event_fetch_start.elapsed();

            let new_from = event.next_from().unwrap_or(pull_from);
            self.sink.process(event, &mut perf_aggregator).await?;
            pull_from = new_from;
        }
    }
}
