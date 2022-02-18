use sea_schema::migration::prelude::*;

use entity::prelude::{Address, AddressColumn, Transaction, TransactionColumn};
use entity::transaction_output::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000005_create_transaction_output_table"
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
                    .col(ColumnDef::new(Column::Payload).binary().not_null())
                    .col(ColumnDef::new(Column::AddressId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_output-address_id")
                            .from(Entity, Column::AddressId)
                            .to(Address, AddressColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::TxId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_output-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::OutputIndex).integer().not_null())
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
