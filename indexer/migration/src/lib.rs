pub use sea_schema::migration::*;

mod m20220210_000001_create_block_table;
mod m20220210_000002_create_transaction_table;
mod m20220211_000001_create_stake_credential_table;
mod m20220211_000002_create_tx_credential_table;
mod m20220211_000003_create_address_table;
mod m20220211_000004_create_address_credential_table;
mod m20220211_000005_create_transaction_output_table;
mod m20220211_000006_create_transaction_input_table;
mod m20220508_000007_create_metadata_table;
mod m20220520_000008_create_native_asset_table;
mod m20220520_000009_create_asset_mint_table;
mod m20220520_000010_create_cip25_entry_table;
mod m20220528_000011_create_plutus_data_hash_table;
mod m20220528_000012_create_plutus_data_table;
mod m20220808_000013_create_transaction_reference_input_table;
mod m20221031_000014_create_dex_table;
mod m20230223_000015_modify_block_table;
mod m20230927_231206_create_stake_delegation_table;

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
            Box::new(m20220508_000007_create_metadata_table::Migration),
            Box::new(m20220520_000008_create_native_asset_table::Migration),
            Box::new(m20220520_000009_create_asset_mint_table::Migration),
            Box::new(m20220520_000010_create_cip25_entry_table::Migration),
            Box::new(m20220528_000011_create_plutus_data_hash_table::Migration),
            Box::new(m20220528_000012_create_plutus_data_table::Migration),
            Box::new(m20220808_000013_create_transaction_reference_input_table::Migration),
            Box::new(m20221031_000014_create_dex_table::Migration),
            Box::new(m20230223_000015_modify_block_table::Migration),
            Box::new(m20230927_231206_create_stake_delegation_table::Migration),
        ]
    }
}
