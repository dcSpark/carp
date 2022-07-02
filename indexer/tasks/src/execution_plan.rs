use std::fs;

use toml::Value;
use tracing_subscriber::prelude::*;

#[derive(Debug)]
pub struct ExecutionPlan(pub toml::value::Table);

impl ExecutionPlan {
    pub fn load_from_file(path: &str) -> ExecutionPlan {
        match &fs::read_to_string(path) {
            Ok(execution_plan_content) => {
                let setting: Result<toml::value::Table, toml::de::Error> =
                    toml::from_str(execution_plan_content);

                ExecutionPlan(setting.unwrap())
            }
            Err(err) => {
                tracing::error!("No execution plan found at {}", path);
                panic!("{}", err);
            }
        }
    }
}

impl From<Vec<&str>> for ExecutionPlan {
    fn from(tasks: Vec<&str>) -> Self {
        let map = tasks
            .into_iter()
            .map(|task| (task.to_string(), Value::Table(toml::value::Table::new())))
            .collect();
        ExecutionPlan(map)
    }
}
