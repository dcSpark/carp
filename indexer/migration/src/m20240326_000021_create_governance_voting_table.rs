use entity::governance_votes::*;
use entity::prelude::{Transaction, TransactionColumn};
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240326_000021_create_governance_vote_table"
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
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .col(ColumnDef::new(Column::Voter).binary())
                    .col(ColumnDef::new(Column::GovActionId).binary())
                    .col(ColumnDef::new(Column::Vote).binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-governance_vote-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("governance_vote-pk")
                            .col(Column::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-governance_vote-voter")
                    .col(Column::Voter)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
