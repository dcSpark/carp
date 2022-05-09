use sea_schema::migration::prelude::*;

use entity::prelude::{Transaction, TransactionColumn};
use entity::transaction_metadata::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220508_000007_create_metadata_table"
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
                            .name("fk-metadata-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Label).binary().not_null())
                    .col(ColumnDef::new(Column::Payload).binary().not_null())
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("metadata-pk")
                            .col(Column::TxId)
                            .col(Column::Label),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-metadata-label")
                    .col(Column::Label)
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
