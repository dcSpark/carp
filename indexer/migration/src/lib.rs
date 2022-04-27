pub use sea_schema::migration::*;

mod m20220210_000001_create_block_table;
mod m20220210_000002_create_transaction_table;
mod m20220211_000001_create_stake_credential_table;
mod m20220211_000002_create_tx_credential_table;
mod m20220211_000003_create_address_table;
mod m20220211_000004_create_address_credential_table;
mod m20220211_000005_create_transaction_output_table;
mod m20220211_000006_create_transaction_input_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220210_000001_create_block_table::Migration),
            Box::new(m20220210_000002_create_transaction_table::Migration),
            Box::new(m20220211_000001_create_stake_credential_table::Migration),
            Box::new(m20220211_000002_create_tx_credential_table::Migration),
            Box::new(m20220211_000003_create_address_table::Migration),
            Box::new(m20220211_000004_create_address_credential_table::Migration),
            Box::new(m20220211_000005_create_transaction_output_table::Migration),
            Box::new(m20220211_000006_create_transaction_input_table::Migration),
        ]
    }
}
