use crate::task_macro::*;
use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
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

async fn handle_dummy(
    _db_tx: &DatabaseTransaction,
    _block: BlockInfo<'_, alonzo::Block>,
) -> Result<(), DbErr> {
    Ok(())
}

carp_task! {
  name ExampleTask;
  era multiera;
  dependencies [];
  read [multiera_txs];
  write [multiera_addresses];
  should_add_task |_block, _properties| {
    true
  };
  execute |_previous_data, task| handle_dummy(
      task.db_tx,
      task.block,
  );
  merge_result |data, _result| {
  };
}
