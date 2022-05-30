use std::fs;

use toml::Value;
use tracing_subscriber::prelude::*;

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
    pub fn load_from_ipfs(url: &str) -> ExecutionPlan {
        // TODO: not ready! just a placeholder
        match &fs::read_to_string(url) {
            Ok(execution_plan_content) => {
                let setting: Result<toml::value::Table, toml::de::Error> =
                    toml::from_str(execution_plan_content);

                ExecutionPlan(setting.unwrap())
            }
            Err(err) => {
                tracing::error!("No execution plan found at {}", url);
                panic!("{}", err);
            }
        }
    }

    pub fn load_execution_plan(path: &str) -> ExecutionPlan {
        if path.starts_with("https://") || path.starts_with("http://"){
            return ExecutionPlan::load_from_ipfs(path);
        } else {
            return ExecutionPlan::load_from_file(path);
        }

    }
}
