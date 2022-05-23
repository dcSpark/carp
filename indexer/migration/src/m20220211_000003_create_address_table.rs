use sea_schema::migration::prelude::*;

use entity::address::*;
use entity::prelude::{Transaction, TransactionColumn};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000003_create_address_table"
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
                    .col(
                        ColumnDef::new(Column::Payload)
                            .binary()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Column::FirstTx).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-address-tx_id")
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
                    .name("index-address-transaction")
                    .col(Column::FirstTx)
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
