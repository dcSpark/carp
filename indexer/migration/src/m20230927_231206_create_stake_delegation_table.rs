use entity::prelude::{StakeCredential, StakeCredentialColumn, Transaction, TransactionColumn};
use entity::stake_delegation::*;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230927_231206_create_stake_delegation_table"
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
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Column::StakeCredential)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .col(ColumnDef::new(Column::PoolCredential).binary())
                    .col(ColumnDef::new(Column::PreviousPool).binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-stake_delegation-credential_id")
                            .from(Entity, Column::StakeCredential)
                            .to(StakeCredential, StakeCredentialColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-stake_delegation-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("stake_delegation_credential-pk")
                            .col(Column::Id),
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
