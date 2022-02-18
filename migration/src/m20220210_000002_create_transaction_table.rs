use sea_schema::migration::prelude::*;

use entity::prelude::{Block, BlockColumn};
use entity::transaction::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220210_000002_create_transaction_table"
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Column::Hash).binary().not_null())
                    .col(ColumnDef::new(Column::BlockId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction-block_id")
                            .from(Entity, Column::BlockId)
                            .to(Block, BlockColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::TxIndex).integer().not_null())
                    .col(ColumnDef::new(Column::Payload).binary().not_null())
                    .col(ColumnDef::new(Column::IsValid).boolean().not_null())
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
