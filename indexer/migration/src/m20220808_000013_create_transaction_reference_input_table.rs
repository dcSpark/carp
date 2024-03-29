use sea_schema::migration::prelude::*;

use entity::prelude::{
    Address, AddressColumn, Transaction, TransactionColumn, TransactionOutput,
    TransactionOutputColumn,
};
use entity::transaction_reference_input::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220808_000013_create_transaction_reference_input_table"
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
                    .col(ColumnDef::new(Column::UtxoId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_reference-input-utxo_id")
                            .from(Entity, Column::UtxoId)
                            .to(TransactionOutput, TransactionOutputColumn::Id)
                            // TODO: sea-query doesn't support RESTRICT DEFERRED
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_reference-input-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::AddressId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_reference-input-address_id")
                            .from(Entity, Column::AddressId)
                            .to(Address, AddressColumn::Id)
                            // TODO: sea-query doesn't support RESTRICT DEFERRED
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::InputIndex).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-transaction_reference_input-transaction_output")
                    .col(Column::UtxoId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-transaction_reference_input-address")
                    .col(Column::AddressId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-transaction_reference_input-transaction")
                    .col(Column::TxId)
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
