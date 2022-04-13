use sea_schema::migration::prelude::*;

use entity::prelude::{StakeCredential, StakeCredentialColumn, Transaction, TransactionColumn};
use entity::tx_credential::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000002_create_tx_credential_table"
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
                        ColumnDef::new(Column::CredentialId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tx_credential-credential_id")
                            .from(Entity, Column::CredentialId)
                            .to(StakeCredential, StakeCredentialColumn::Id),
                    )
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tx_credential-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Relation).integer().not_null())
                    // Note: the 3-tuple is the primary key
                    // The order of the key here matters since it affects how the generated index performs
                    // https://stackoverflow.com/a/11352543
                    // Since all queries that include this table will include joins on <TxId, CredentialId>
                    // the performance should still be good
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("tx_credential-pk")
                            .col(Column::TxId)
                            .col(Column::CredentialId)
                            .col(Column::Relation),
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
