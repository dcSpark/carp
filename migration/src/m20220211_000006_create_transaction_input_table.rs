use sea_schema::migration::prelude::*;

use entity::prelude::{Transaction, TransactionColumn, TransactionOutput, TransactionOutputColumn};
use entity::transaction_input::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000006_create_transaction_input_table"
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
                    .col(ColumnDef::new(Column::UtxoId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_input-utxo_id")
                            .from(Entity, Column::UtxoId)
                            .to(TransactionOutput, TransactionOutputColumn::Id),
                    )
                    .col(ColumnDef::new(Column::TxId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_input-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id),
                    )
                    .col(ColumnDef::new(Column::InputIndex).integer().not_null())
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
