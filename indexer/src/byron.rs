use crate::tasks::byron::byron_outputs::ByronOutputTask;
use crate::tasks::byron::byron_txs::ByronTransactionTask;
use crate::{perf_aggregator::PerfAggregator, tasks::byron::byron_inputs::ByronInputTask};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::primitives::byron::{self};
use shred::{DispatcherBuilder, World};
use tokio::runtime::Handle;

pub async fn process_byron_block(
    perf_aggregator: &mut PerfAggregator,
    time_counter: &mut std::time::Instant,
    txn: &DatabaseTransaction,
    block: (&byron::Block, &BlockModel),
) -> Result<(), DbErr> {
    let handle = Handle::current();

    let mut world = World::empty();
    let mut dispatcher = DispatcherBuilder::new()
        .with(
            ByronTransactionTask {
                db_tx: txn,
                block,
                handle: &handle,
            },
            "ByronTransactionTask",
            &[],
        )
        .with(
            ByronOutputTask {
                db_tx: txn,
                block,
                handle: &handle,
            },
            "ByronOutputTask",
            &["ByronTransactionTask"],
        )
        .with(
            ByronInputTask {
                db_tx: txn,
                block,
                handle: &handle,
            },
            "ByronInputTask",
            &["ByronOutputTask"],
        )
        .build();
    dispatcher.setup(&mut world);
    dispatcher.dispatch(&world);

    Ok(())
}
