use crate::task_macro::*;
use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, PrerunResult, TaskBuilder,
        TaskRegistryEntry,
    },
    utils::TaskPerfAggregator,
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
use pallas::ledger::primitives::alonzo::{self};
use paste::paste;
use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone)]
pub struct ExampleTaskPrerunData();

async fn handle_dummy(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, alonzo::Block>,
) -> Result<(), DbErr> {
    Ok(())
}

carp_task! {
  name ExampleTask;
  era multiera;
  dependencies [];
  read [multiera_txs];
  write [multiera_addresses];
  should_add_task |block, properties| -> ExampleTaskPrerunData {
    PrerunResult::RunTaskWith(ExampleTaskPrerunData())
  };
  execute |previous_data, task| handle_dummy(
      task.db_tx,
      task.block,
  );
  merge_result |data, result| {
  };
}
