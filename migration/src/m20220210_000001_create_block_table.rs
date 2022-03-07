use sea_schema::migration::prelude::*;

use entity::block::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220210_000001_create_block_table"
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
                    .col(ColumnDef::new(Column::Era).integer().not_null())
                    .col(ColumnDef::new(Column::Hash).binary().not_null())
                    .col(ColumnDef::new(Column::Height).big_integer().not_null())
                    .col(ColumnDef::new(Column::Epoch).integer().not_null())
                    .col(ColumnDef::new(Column::Slot).big_integer().not_null())
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
