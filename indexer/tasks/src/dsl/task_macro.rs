pub(crate) use super::execution_context::*;
pub use crate::utils::find_task_registry_entry;
pub use crate::{
    dsl::database_task::{
        BlockGlobalInfo, BlockInfo, ByronTaskRegistryEntry, DatabaseTaskMeta,
        GenesisTaskRegistryEntry, MultieraTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    era_common::AddressInBlock,
    utils::TaskPerfAggregator,
};
pub use cml_chain::genesis::byron::config::GenesisData;
pub use paste::paste;
pub use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
pub use std::sync::{Arc, Mutex};

macro_rules! era_to_block {
    (genesis) => {
        GenesisData
    };
    (byron) => {
        cml_multi_era::MultiEraBlock
    };
    (multiera) => {
        cml_multi_era::MultiEraBlock
    };
}

macro_rules! era_to_block_info {
    (genesis) => {
        BlockGlobalInfo
    };
    (byron) => {
        BlockGlobalInfo
    };
    (multiera) => {
        BlockGlobalInfo
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "build_markdown_task")] {
        macro_rules! era_to_registry {
            (genesis $task_builder:expr) => {
                TaskMarkdownRegistryEntry::Genesis(GenesisTaskMarkdownRegistryEntry {
                    builder: &$task_builder,
                })
            };
            (byron $task_builder:expr) => {
                TaskMarkdownRegistryEntry::Byron(ByronTaskMarkdownRegistryEntry {
                    builder: &$task_builder,
                })
            };
            (multiera $task_builder:expr) => {
                TaskMarkdownRegistryEntry::Multiera(MultieraTaskMarkdownRegistryEntry {
                    builder: &$task_builder,
                })
            };
        }

        macro_rules! carp_task {
            (
                name $name:ident;
                configuration $config:ty;
                doc $doc:expr;
                era $era:ident;
                dependencies [ $( $dep:ty ),* ];
                read [ $( $read_name:ident ),* ];
                write [ $( $write_name:ident ),* ];
                should_add_task |$block:ident, $properties:ident| { $($should_add_task:tt)* };
                execute |$previous_data:ident, $task:ident| $execute:expr;
                merge_result |$next_data:ident, $execution_result:ident| $merge_closure:expr;
              ) => {
                use markdown_gen::markdown::Markdown;
                use markdown_gen::markdown::AsMarkdown;
                use markdown_gen::markdown::List;
                use std::fs::File;
                use urlencoding::encode;
                pub use crate::{
                    dsl::markdown_task::*,
                };

                pub struct $name {}

                impl<'a> MarkdownTaskMeta for $name {
                    const TASK_NAME: &'static str = stringify!($name);
                    const DOC: &'static str = stringify!($doc);
                    const ERA: &'static str = stringify!($era);
                    const READ_FROM: &'static [&'static str] = &[
                        $(
                            stringify!($read_name)
                        ),*
                    ];
                    const WRITE_TO: &'static [&'static str] = &[
                        $(
                            stringify!($write_name)
                        ),*
                    ];
                    const DEPENDENCIES: &'static [&'static str] = &[
                        $(
                            nameof::name_of_type!($dep)
                        ),*
                    ];
                }

                paste! { struct [< $name Builder >]; }
                impl<'a> TaskMarkdownBuilder for paste! { [< $name Builder >] } {
                    fn get_name(&self) -> &'static str {
                        $name::TASK_NAME
                    }
                    fn get_doc(&self) -> &'static str {
                        $name::DOC
                    }
                    fn get_era(&self) -> &'static str {
                        $name::ERA
                    }
                    fn get_reads(&self) -> &'static [&'static str] {
                        $name::READ_FROM
                    }
                    fn get_writes(&self) -> &'static [&'static str] {
                        $name::WRITE_TO
                    }
                    fn get_dependencies(&self) -> &'static [&'static str] {
                        $name::DEPENDENCIES
                    }

                    fn generate_docs(
                        &self,
                        md: &mut Markdown<File>,
                    ) {
                        md.write(self.get_name().heading(1)).unwrap();

                        md.write(self.get_doc()[1..self.get_doc().len()-1].paragraph()).unwrap();

                        let config = include_str!(concat!("../config/", stringify!($config), ".rs"));
                        let config_docs = format!("
<details>
    <summary>Configuration</summary>

```rust
{}
```
</details>
", config);
                        md.write_raw(config_docs.paragraph()).unwrap();

                        md.write("Era".heading(2)).unwrap();
                        md.write(self.get_era().code()).unwrap();

                        let dependencies = self.get_dependencies();
                        let dep_strs: Vec<String> = dependencies.iter().map(|dep| format!("./{}", encode(dep))).collect();
                        if !dependencies.is_empty() {
                            md.write("Dependencies".heading(2)).unwrap();
                            let mut dep_list = List::new(false);
                            for (i, dep) in dependencies.iter().enumerate() {
                                dep_list = dep_list.item(dep.link_to(&dep_strs[i]));
                            }
                            md.write_raw(dep_list).unwrap();
                            md.write("\n").unwrap();
                        }

                        md.write("Data accessed".heading(2)).unwrap();

                        let reads = self.get_reads();
                        if !reads.is_empty() {
                            md.write("Reads from".heading(4)).unwrap();
                            let mut read_list = List::new(false);
                            for &read in reads {
                                read_list = read_list.item(read.code());
                            }
                            md.write(read_list).unwrap();
                            md.write("\n").unwrap();
                        }

                        let writes = self.get_writes();
                        if !writes.is_empty() {
                            md.write("Writes to".heading(4)).unwrap();
                            let mut write_list = List::new(false);
                            for &write in writes {
                                write_list = write_list.item(write.code());
                            }
                            md.write(write_list).unwrap();
                            md.write("\n").unwrap();
                        }

                        md.write("Full source".heading(2)).unwrap();

                        let source_url = format!("{}{}", "https://github.com/dcSpark/carp/tree/main/indexer/",file!());
                        md.write_raw("source".link_to(&source_url)).unwrap();
                    }
                }

                paste! {
                    inventory::submit! {
                        era_to_registry! { $era [< $name Builder >] }
                    }
                }
            };
        }
    } else if #[cfg(feature = "build_rust_task")] {

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
              configuration $config:ty;
              doc $doc:expr;
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
                    pub block: BlockInfo<'a, era_to_block!($era), BlockGlobalInfo>,
                    pub handle: &'a tokio::runtime::Handle,
                    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
                    pub config: $config,
                }

                impl<'a> DatabaseTaskMeta<'a, era_to_block!($era), era_to_block_info!($era), $config> for $name<'a> {
                    const TASK_NAME: &'static str = stringify!($name);
                    const DEPENDENCIES: &'static [&'static str] = &[
                        $(
                            nameof::name_of_type!($dep)
                        ),*
                    ];

                    fn new(
                        db_tx: &'a DatabaseTransaction,
                        block: BlockInfo<'a, era_to_block!($era), BlockGlobalInfo>,
                        handle: &'a tokio::runtime::Handle,
                        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
                        config: &$config,
                    ) -> Self {
                        Self {
                            db_tx,
                            block,
                            handle,
                            perf_aggregator,
                            config: config.clone(),
                        }
                    }

                    fn get_configuration(&self) -> &$config {
                        &self.config
                    }

                    fn should_add_task(
                        $block: BlockInfo<'a, era_to_block!($era), BlockGlobalInfo>,
                        $properties: &toml::value::Value,
                    ) -> bool {
                        $($should_add_task)*
                    }
                }

                paste! { struct [< $name Builder >]; }
                impl<'a> TaskBuilder<'a, era_to_block!($era), era_to_block_info!($era)> for paste! { [< $name Builder >] } {
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
                        block: BlockInfo<'a, era_to_block!($era), BlockGlobalInfo>,
                        handle: &'a tokio::runtime::Handle,
                        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
                        configuration: &toml::value::Value,
                    ) -> bool {
                      match &$name::should_add_task(block, configuration) {
                        false => false,
                        true => {
                          let config: $config = configuration.clone().try_into::<$config>().unwrap();
                          let task = $name::new(db_tx, block, handle, perf_aggregator, &config);

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
    } else {
        compile_error!("You need to set a feature to specify how to compile Carp tasks");
    }
}

pub(crate) use carp_task;
pub(crate) use era_to_block;
pub(crate) use era_to_block_info;
pub(crate) use era_to_registry;
