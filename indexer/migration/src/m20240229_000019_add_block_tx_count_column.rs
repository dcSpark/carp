use sea_schema::migration::prelude::*;

use entity::block::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240229_000019_add_block_tx_count_column"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Entity)
                    .add_column(ColumnDef::new(Column::TxCount).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Entity)
                    .drop_column(Column::TxCount)
                    .to_owned(),
            )
            .await
    }
}
