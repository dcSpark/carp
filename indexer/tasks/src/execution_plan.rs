use std::fs;

use anyhow::anyhow;
use toml::Value;
use tracing_subscriber::prelude::*;

pub struct ExecutionPlan(pub toml::value::Table);

impl ExecutionPlan {
    pub fn load_from_file(path: &str) -> anyhow::Result<ExecutionPlan> {
        match &fs::read_to_string(path) {
            Ok(execution_plan_content) => {
                let setting: Result<toml::value::Table, toml::de::Error> =
                    toml::from_str(execution_plan_content);

                Ok(ExecutionPlan(setting.unwrap()))
            }
            Err(err) => {
                tracing::error!("No execution plan found at {}", path);
                Err(anyhow!(
                    "No execution plan found at {}, error: {}",
                    path,
                    err
                ))
            }
        }
    }
}
