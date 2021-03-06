use sea_schema::migration::prelude::*;

use entity::native_asset::*;
use entity::prelude::{Transaction, TransactionColumn};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220520_000008_create_native_asset_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Column::PolicyId).binary().not_null())
                    .col(ColumnDef::new(Column::AssetName).binary().not_null())
                    .col(ColumnDef::new(Column::Cip14Fingerprint).binary().not_null())
                    .col(ColumnDef::new(Column::FirstTx).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-native_asset-tx_id")
                            .from(Entity, Column::FirstTx)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-native_asset-transaction")
                    .col(Column::FirstTx)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-native_asset-pair")
                    .col(Column::PolicyId)
                    .col(Column::AssetName)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-native_asset-fingerprint")
                    .col(Column::Cip14Fingerprint)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-native_asset_name")
                    .col(Column::AssetName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
