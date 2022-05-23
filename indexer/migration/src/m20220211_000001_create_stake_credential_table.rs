use sea_schema::migration::prelude::*;

use entity::prelude::{Transaction, TransactionColumn};
use entity::stake_credential::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000001_create_stake_credential_table"
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
                        ColumnDef::new(Column::Credential)
                            .binary()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Column::FirstTx).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-stake_credential-tx_id")
                            .from(Entity, Column::FirstTx)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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
