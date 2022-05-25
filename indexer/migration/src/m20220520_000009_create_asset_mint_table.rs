use sea_schema::migration::prelude::*;

use entity::asset_mint::*;
use entity::prelude::{NativeAsset, NativeAssetColumn, Transaction, TransactionColumn};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220520_000009_create_asset_mint_table"
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
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-asset_mint-transaction_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::AssetId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-asset_mint-asset_id")
                            .from(Entity, Column::AssetId)
                            .to(NativeAsset, NativeAssetColumn::Id)
                            // TODO: sea-query doesn't support RESTRICT DEFERRED
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Amount).big_integer().not_null())
                    // Note: the 2-tuple is the primary key
                    // This creates an index on <TxId> and <TxId, AssetId> (https://stackoverflow.com/a/11352543)
                    // so we also need to explicitly create an index on AssetId
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("asset_mint-pk")
                            .col(Column::TxId)
                            .col(Column::AssetId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-asset_mint-native_asset")
                    .col(Column::AssetId)
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
