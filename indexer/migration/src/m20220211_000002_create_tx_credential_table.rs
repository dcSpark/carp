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
                            .to(StakeCredential, StakeCredentialColumn::Id)
                            // TODO: sea-query doesn't support RESTRICT DEFERRED
                            .on_delete(ForeignKeyAction::Cascade),
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
                    // Note: the 2-tuple is the primary key
                    // This creates an index on <TxId> and <TxId, CredentialId> (https://stackoverflow.com/a/11352543)
                    // so we also need to explicitly create an index on CredentialId
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("tx_credential-pk")
                            .col(Column::TxId)
                            .col(Column::CredentialId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-tx_credential-credential")
                    .col(Column::CredentialId)
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
