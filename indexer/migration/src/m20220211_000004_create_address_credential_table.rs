use sea_schema::migration::prelude::*;

use entity::address_credential::*;
use entity::prelude::{Address, AddressColumn, StakeCredential, StakeCredentialColumn};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220211_000004_create_address_credential_table"
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
                    .col(ColumnDef::new(Column::AddressId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-address_credential-address_id")
                            .from(Entity, Column::AddressId)
                            .to(Address, AddressColumn::Id),
                    )
                    .col(
                        ColumnDef::new(Column::CredentialId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-address_credential-credential_id")
                            .from(Entity, Column::CredentialId)
                            .to(StakeCredential, StakeCredentialColumn::Id),
                    )
                    .col(ColumnDef::new(Column::Relation).integer().not_null())
                    // Note: the 3-tuple is the primary key
                    // The order of the key here matters since it affects how the generated index performs
                    // https://stackoverflow.com/a/11352543
                    // so we also need to explicitly create an index on CredentialId
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("address_credential-pk")
                            .col(Column::AddressId)
                            .col(Column::CredentialId)
                            .col(Column::Relation),
                    )
                    .index(
                        Index::create()
                            .name("index-address_credential-credential")
                            .col(Column::CredentialId),
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
