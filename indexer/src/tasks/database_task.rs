use crate::tasks::utils::TaskPerfAggregator;
use entity::{prelude::*, sea_orm::DatabaseTransaction};
use std::sync::{Arc, Mutex};

pub trait DatabaseTask<'a, BlockType> {
    fn new(
        db_tx: &'a DatabaseTransaction,
        block: (&'a BlockType, &'a BlockModel),
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Self;
}
