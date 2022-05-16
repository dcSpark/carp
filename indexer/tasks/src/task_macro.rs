pub use crate::utils::find_task_registry_entry;
pub use crate::{
    database_task::{
        BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta, GenesisTaskRegistryEntry,
        MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    era_common::AddressInBlock,
    utils::TaskPerfAggregator,
};
pub use cardano_multiplatform_lib::genesis::byron::config::GenesisData;
pub use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction},
};
pub use pallas::ledger::primitives::alonzo::{self};
pub use pallas::ledger::primitives::byron::{self};
pub use paste::paste;
pub use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
pub use std::collections::BTreeMap;
pub use std::sync::{Arc, Mutex};

#[macro_export]
macro_rules! data_to_type {
  // genesis
  (genesis_txs) => { Vec<TransactionModel> };
  (genesis_addresses) => { Vec<AddressModel> };
  (genesis_outputs) => { Vec<TransactionOutputModel> };

  // byron
  (byron_txs) => { Vec<TransactionModel> };
  (byron_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };
  (byron_inputs) => { Vec<TransactionInputModel> };
  (byron_outputs) => { Vec<TransactionOutputModel> };

  // multiera
  (multiera_txs) => { Vec<TransactionModel> };
  (vkey_relation_map) => { RelationMap };
  (multiera_queued_addresses_relations) => { BTreeSet<QueuedAddressCredentialRelation> };
  (multiera_stake_credential) => { BTreeMap<Vec<u8>, StakeCredentialModel> };
  (multiera_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };
  (multiera_metadata) => { Vec<TransactionMetadataModel> };
  (multiera_outputs) => { Vec<TransactionOutputModel> };
  (multiera_used_inputs) => { Vec<TransactionInputModel> };
}

macro_rules! era_to_block {
    (genesis) => {
        GenesisData
    };
    (byron) => {
        byron::Block
    };
    (multiera) => {
        alonzo::Block
    };
}

macro_rules! era_to_registry {
    (genesis $task_builder:expr) => {
        TaskRegistryEntry::Genesis(GenesisTaskRegistryEntry {
            builder: &$task_builder,
        })
    };
    (byron $task_builder:expr) => {
        TaskRegistryEntry::Byron(ByronTaskRegistryEntry {
            builder: &$task_builder,
        })
    };
    (multiera $task_builder:expr) => {
        TaskRegistryEntry::Multiera(MultieraTaskRegistryEntry {
            builder: &$task_builder,
        })
    };
}

macro_rules! carp_task {
    (
      name $name:ident;
      era $era:ident;
      dependencies [ $( $dep:ty ),* ];
      read [ $( $read_name:ident ),* ];
      write [ $( $write_name:ident ),* ];
      should_add_task |$block:ident, $properties:ident| { $($should_add_task:tt)* };
      execute |$previous_data:ident, $task:ident| $execute:expr;
      merge_result |$next_data:ident, $execution_result:ident| $merge_closure:expr;
    ) => {
        #[derive(SystemData)]
        pub struct Data<'a> {
            $(
                pub $read_name : Read<'a, data_to_type! { $read_name } >,
            )*
            $(
                pub $write_name : Write<'a, data_to_type! { $write_name } >,
            )*
        }

        pub struct $name<'a> {
            pub db_tx: &'a DatabaseTransaction,
            pub block: BlockInfo<'a, era_to_block!($era)>,
            pub handle: &'a tokio::runtime::Handle,
            pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        }

        impl<'a> DatabaseTaskMeta<'a, era_to_block!($era)> for $name<'a> {
            const TASK_NAME: &'static str = stringify!($name);
            const DEPENDENCIES: &'static [&'static str] = &[
                $(
                    nameof::name_of_type!($dep)
                ),*
            ];

            fn new(
                db_tx: &'a DatabaseTransaction,
                block: BlockInfo<'a, era_to_block!($era)>,
                handle: &'a tokio::runtime::Handle,
                perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
            ) -> Self {
                Self {
                    db_tx,
                    block,
                    handle,
                    perf_aggregator,
                }
            }

            fn should_add_task(
                $block: BlockInfo<'a, era_to_block!($era)>,
                $properties: &ini::Properties,
            ) -> bool {
                $($should_add_task)*
            }
        }

        paste! { struct [< $name Builder >]; }
        impl<'a> TaskBuilder<'a, era_to_block!($era)> for paste! { [< $name Builder >] } {
            fn get_name(&self) -> &'static str {
                $name::TASK_NAME
            }
            fn get_dependencies(&self) -> &'static [&'static str] {
                $name::DEPENDENCIES
            }

            fn maybe_add_task<'c>(
                &self,
                dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
                db_tx: &'a DatabaseTransaction,
                block: BlockInfo<'a, era_to_block!($era)>,
                handle: &'a tokio::runtime::Handle,
                perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
                properties: &ini::Properties,
            ) -> bool {
              match &$name::should_add_task(block, properties) {
                false => false,
                true => {
                  let task = $name::new(db_tx, block, handle, perf_aggregator);

                  // 1) Check that all dependencies are registered tasks
                  for dep in self.get_dependencies().iter() {
                    if find_task_registry_entry(dep).is_none() {
                        panic!("Could not find task named {} in dependencies of {}", dep, self.get_name());
                    }
                  }
                  // 2) Filter out any dependency that got skipped
                  let filtered_deps: Vec<&str> = self.get_dependencies().iter()
                    .map(|&dep| dep)
                    .filter(|&dep| dispatcher_builder.has_system(dep))
                    .collect();
                   // check all tasks are registered, then remove skipped ones
                  dispatcher_builder.add(task, self.get_name(), filtered_deps.as_slice());
                  true
                }
              }
            }
        }

        paste! {
            inventory::submit! {
                era_to_registry! { $era [< $name Builder >] }
            }
        }

        impl<'a> System<'a> for $name<'_> {
            type SystemData = Data<'a>;

            #[allow(unused_mut)]
            fn run(&mut self, mut $previous_data: Data<'a>) {
                let time_counter = std::time::Instant::now();

                let $task = &self;
                let $execution_result = self
                    .handle
                    .block_on($execute)
                    .unwrap();
                $merge_closure;

                self.perf_aggregator
                    .lock()
                    .unwrap()
                    .update(Self::TASK_NAME, time_counter.elapsed());
            }
        }
    };
}

pub(crate) use carp_task;
pub(crate) use data_to_type;
pub(crate) use era_to_block;
pub(crate) use era_to_registry;
