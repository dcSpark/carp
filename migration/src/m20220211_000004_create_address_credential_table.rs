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
                    .col(
                        ColumnDef::new(Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
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
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-address_credential-address")
                    .col(Column::AddressId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-address_credential-stake_credential")
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
