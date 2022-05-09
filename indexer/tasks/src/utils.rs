use std::{collections::BTreeMap, time::Duration};

use cryptoxide::blake2b::Blake2b;

use super::database_task::TaskRegistryEntry;

pub fn blake2b256(data: &[u8]) -> [u8; 32] {
    let mut out = [0; 32];
    Blake2b::blake2b(&mut out, data, &[]);
    out
}

#[derive(Default, Debug)]
pub struct TaskPerfAggregator(pub BTreeMap<&'static str, Duration>);
impl TaskPerfAggregator {
    const TOTAL_TIME: &'static str = "TotalPlanExecutionTime";

    pub fn update(&mut self, task: &'static str, duration: Duration) {
        self.0
            .entry(task)
            .and_modify(|old| *old += duration)
            .or_insert_with(|| duration);
    }

    pub fn get_total(&mut self) -> Duration {
        self.0
            .get(TaskPerfAggregator::TOTAL_TIME)
            .copied()
            .unwrap_or_else(|| Duration::new(0, 0))
    }
    pub fn add_to_total(&mut self, duration: &Duration) {
        self.0.insert(TaskPerfAggregator::TOTAL_TIME, *duration);
    }
}

pub fn find_task_registry_entry(task_name: &str) -> Option<TaskRegistryEntry> {
    for registry_entry in inventory::iter::<TaskRegistryEntry> {
        match registry_entry {
            TaskRegistryEntry::Byron(entry) => {
                if entry.builder.get_name() == task_name {
                    return Some(*registry_entry);
                }
            }
            TaskRegistryEntry::Multiera(entry) => {
                if entry.builder.get_name() == task_name {
                    return Some(*registry_entry);
                }
            }
        }
    }
    None
}
