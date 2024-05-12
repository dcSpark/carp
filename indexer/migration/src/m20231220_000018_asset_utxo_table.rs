use entity::{asset_utxos::*, prelude};
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        // NOTE: this is wrong - it shouldn't end with `.rs`
        //       but fixing this would break existing DBs since the migration name would no longer match
        //       so since it doesn't harm anything, we just leave it as-is
        "m20231220_000018_asset_utxos.rs"
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
                    .col(ColumnDef::new(Column::AssetId).big_integer().not_null())
                    .col(ColumnDef::new(Column::UtxoId).big_integer().not_null())
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .col(ColumnDef::new(Column::Amount).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-asset_utxo-asset_id")
                            .from(Entity, Column::AssetId)
                            .to(prelude::NativeAsset, prelude::NativeAssetColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-asset_utxo-tx_id")
                            .from(Entity, Column::TxId)
                            .to(prelude::Transaction, prelude::TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-asset_utxo-utxo_id")
                            .from(Entity, Column::UtxoId)
                            .to(
                                prelude::TransactionOutput,
                                prelude::TransactionOutputColumn::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .table(Entity)
                            .name("asset_utxo-pk")
                            .col(Column::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-asset_utxo-transaction")
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
