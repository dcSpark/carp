use sea_schema::migration::prelude::*;

use entity::dex_mean_price::*;
use entity::prelude::{
    Address, AddressColumn, NativeAsset, NativeAssetColumn, Transaction, TransactionColumn,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20221020_000014_create_dex_mean_price_table"
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
                    .col(ColumnDef::new(Column::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-dex_mean_price-tx_id")
                            .from(Entity, Column::TxId)
                            .to(Transaction, TransactionColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::AddressId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-dex_mean_price-address_id")
                            .from(Entity, Column::AddressId)
                            .to(Address, AddressColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Dex).big_integer().not_null())
                    .col(ColumnDef::new(Column::Asset1Id).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-dex_mean_price-asset1_id")
                            .from(Entity, Column::Asset1Id)
                            .to(NativeAsset, NativeAssetColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Asset2Id).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-dex_mean_price-asset2_id")
                            .from(Entity, Column::Asset2Id)
                            .to(NativeAsset, NativeAssetColumn::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Column::Amount1).big_unsigned().not_null())
                    .col(ColumnDef::new(Column::Amount2).big_unsigned().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Entity)
                    .name("index-dex_mean_price-address-native_asset1-native_asset2-transaction")
                    .col(Column::AddressId)
                    .col(Column::Asset1Id)
                    .col(Column::Asset2Id)
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
