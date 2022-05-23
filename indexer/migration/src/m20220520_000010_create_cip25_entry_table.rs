use sea_schema::migration::prelude::*;

use entity::cip25_entry::*;
use entity::prelude::{
    NativeAsset, NativeAssetColumn, TransactionMetadata, TransactionMetadataColumn,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220520_000010_create_cip25_entry_table"
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
                            .primary_key()
                            .auto_increment()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Column::MetadataId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cip25_entry-metadata")
                            .from(Entity, Column::MetadataId)
                            .to(TransactionMetadata, TransactionMetadataColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::AssetId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cip25_entry-asset_id")
                            .from(Entity, Column::AssetId)
                            .to(NativeAsset, NativeAssetColumn::Id),
                    )
                    .col(ColumnDef::new(Column::Version).text().not_null())
                    .col(ColumnDef::new(Column::Payload).binary().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-cip25_entry-metadata")
                    .col(Column::MetadataId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-cip25_entry-native_asset")
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
