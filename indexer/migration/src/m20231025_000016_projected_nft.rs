use sea_schema::migration::prelude::*;

use entity::projected_nft::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20231025_000016_projected_nft"
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
                    .col(ColumnDef::new(Column::UtxoId).big_integer())
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .col(ColumnDef::new(Column::Asset).text().not_null())
                    .col(ColumnDef::new(Column::Amount).big_integer().not_null())
                    .col(ColumnDef::new(Column::Operation).integer())
                    .col(ColumnDef::new(Column::PlutusDatum).binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-projected_nft-tx_id")
                            .from(Entity, Column::TxId)
                            .to(
                                entity::prelude::Transaction,
                                entity::prelude::TransactionColumn::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-projected_nft-utxo_id")
                            .from(Entity, Column::UtxoId)
                            .to(
                                entity::prelude::TransactionOutput,
                                entity::prelude::TransactionOutputColumn::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("projected_nft-pk")
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
