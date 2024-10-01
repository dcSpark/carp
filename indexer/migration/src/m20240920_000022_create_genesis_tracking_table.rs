use sea_schema::migration::prelude::*;

use entity::genesis::*;
use entity::prelude::{Block, BlockColumn};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240920_000022_create_genesis_tracking_table"
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
                        ColumnDef::new(Column::Era)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Column::BlockId).integer().not_null())
                    .col(ColumnDef::new(Column::BlockHeight).integer().not_null())
                    .col(ColumnDef::new(Column::FirstSlot).big_integer().not_null())
                    .col(ColumnDef::new(Column::StartEpoch).big_integer().not_null())
                    .col(
                        ColumnDef::new(Column::EpochLengthSeconds)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction-block_id")
                            .from(Entity, Column::BlockId)
                            .to(Block, BlockColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
