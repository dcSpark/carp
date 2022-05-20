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
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .col(ColumnDef::new(Column::Label).binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cip25_entry-metadata")
                            .from(Entity, Column::TxId)
                            .from(Entity, Column::Label)
                            .to(TransactionMetadata, TransactionMetadataColumn::TxId)
                            .to(TransactionMetadata, TransactionMetadataColumn::Label)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Column::NativeAssetId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cip25_entry-asset_id")
                            .from(Entity, Column::NativeAssetId)
                            .to(NativeAsset, NativeAssetColumn::Id),
                    )
                    // Note: the 3-tuple is the primary key
                    // This creates an index on <TxId>, <TxId, Label>, <TxId, Label, NativeAssetId> (https://stackoverflow.com/a/11352543)
                    // so we also need to explicitly create an index on NativeAssetId
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("cip25_entry-pk")
                            .col(Column::TxId)
                            .col(Column::Label)
                            .col(Column::NativeAssetId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-cip25_entry-native_asset")
                    .col(Column::NativeAssetId)
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
