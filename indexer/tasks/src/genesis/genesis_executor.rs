use std::sync::{Arc, Mutex};

use crate::dsl::database_task::BlockInfo;
use crate::dsl::database_task::TaskRegistryEntry;
use crate::execution_plan::ExecutionPlan;
use crate::utils::find_task_registry_entry;
use crate::utils::TaskPerfAggregator;
use cardano_multiplatform_lib::genesis::byron::config::GenesisData;
use entity::sea_orm::{prelude::*, DatabaseTransaction};
use shred::{DispatcherBuilder, World};
use tokio::runtime::Handle;

pub async fn process_genesis_block(
    txn: &DatabaseTransaction,
    block_info: BlockInfo<'_, GenesisData>,
    exec_plan: &ExecutionPlan,
    perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
) -> Result<(), DbErr> {
    let ep_start_time = std::time::Instant::now();

    let handle = Handle::current();

    let mut world = World::empty();

    let mut dispatcher_builder = DispatcherBuilder::new();

    for (task_name, val) in exec_plan.0.iter() {
        if let toml::value::Value::Table(_task_props) = val {
            let entry = find_task_registry_entry(task_name);
            match &entry {
                None => {
                    panic!("Could not find task named {}", task_name);
                }
                Some(task) => {
                    if let TaskRegistryEntry::Genesis(entry) = task {
                        entry.builder.maybe_add_task(
                            &mut dispatcher_builder,
                            txn,
                            block_info,
                            &handle,
                            perf_aggregator.clone(),
                            val,
                        );
                    }
                }
            }
        }
    }

    if !dispatcher_builder.is_empty() {
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);
        dispatcher.dispatch(&world);
    }

    perf_aggregator
        .lock()
        .unwrap()
        .add_to_total(&ep_start_time.elapsed());

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::dsl::database_task::BlockGlobalInfo;
    use cardano_multiplatform_lib::crypto::BlockHeaderHash;
    use cardano_multiplatform_lib::fees::LinearFee;
    use cardano_multiplatform_lib::utils::BigNum;
    use core::default::Default;
    use entity::block::EraValue;
    use entity::sea_orm::{MockDatabaseConnection, MockDatabaseConnector, TransactionTrait};
    use std::time::SystemTime;

    async fn mock_database_conn() -> DatabaseConnection {
        let postgres_url = "postgresql://carp:password@localhost:5432/carp_mainnet";
        MockDatabaseConnector::connect(&postgres_url).await.unwrap()
    }

    type OwnedBlockInfo = (String, GenesisData, BlockGlobalInfo);

    fn mock_block_info() -> OwnedBlockInfo {
        let cbor = "".to_string();
        let block_type = GenesisData {
            genesis_prev: BlockHeaderHash::from([0; 32]),
            epoch_stability_depth: 0,
            start_time: SystemTime::UNIX_EPOCH,
            slot_duration: Default::default(),
            protocol_magic: Default::default(),
            fee_policy: LinearFee {
                constant: BigNum::from(0),
                coefficient: BigNum::from(0),
            },
            avvm_distr: Default::default(),
            non_avvm_balances: Default::default(),
            boot_stakeholders: Default::default(),
        };
        let block_global_data = BlockGlobalInfo {
            era: EraValue::Byron,
            epoch: None,
            epoch_slot: None,
        };
        (cbor, block_type, block_global_data)
    }

    fn mock_exec_plan() -> Arc<ExecutionPlan> {
        Arc::new(ExecutionPlan(Default::default()))
    }

    fn mock_perf_aggregator() -> Arc<Mutex<TaskPerfAggregator>> {
        Default::default()
    }

    async fn wrap_process_genesis_block(
        txn: &DatabaseTransaction,
        owned_block_info: OwnedBlockInfo,
        exec_plan: Arc<ExecutionPlan>,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Result<(), DbErr> {
        let block_info = (
            owned_block_info.0.as_str(),
            &owned_block_info.1,
            &owned_block_info.2,
        );
        process_genesis_block(txn, block_info, &exec_plan, perf_aggregator.clone())
            .await
            .unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn process_genesis_block__sanity_check() {
        let conn = mock_database_conn().await;
        let block_info = mock_block_info();
        let exec_plan = mock_exec_plan();
        let perf_aggregator = mock_perf_aggregator();

        conn.transaction(|db_tx| {
            Box::pin(wrap_process_genesis_block(
                db_tx,
                block_info,
                exec_plan.clone(),
                perf_aggregator.clone(),
            ))
        })
        .await
        .unwrap();
    }
}
