use std::sync::Arc;
use dcspark_blockchain_source::{Event, PullFrom, Source};
use dcspark_core::BlockNumber;
use migration::async_std::sync::Mutex;
use tasks::utils::TaskPerfAggregator;
use crate::perf_aggregator::PerfAggregator;
use crate::sink::Sink;

pub struct FetchEngine<EventType, SourceType: Source<From = Option<BlockNumber>, Event = Event<EventType>>, SinkType: Sink<Event = Event<EventType>>> {
    source: SourceType,
    sink: SinkType,
}

impl <EventType, SourceType: Source<From = Option<BlockNumber>, Event = Event<EventType>>, SinkType: Sink<Event = Event<EventType>>> FetchEngine<EventType, SourceType, SinkType> {
    pub fn new(source: SourceType, sink: SinkType) -> FetchEngine<EventType, SourceType, SinkType> {
        Self {
            source,
            sink,
        }
    }

    pub async fn fetch_and_process(&mut self, from: Option<BlockNumber>) -> anyhow::Result<()> {
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

            let new_block_number = event.info.block_number.clone();
            self.sink.process(event)?;
            pull_from = Some(new_block_number);
        }
    }
}