use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Cip25Entry")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub tx_id: i64,
    #[sea_orm(primary_key)]
    pub label: Vec<u8>, // little-endian u64 ([u8; 8]) (https://github.com/launchbadge/sqlx/issues/1374)
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub native_asset_id: i64,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::native_asset::Entity",
        from = "Column::NativeAssetId",
        to = "super::native_asset::Column::Id"
    )]
    NativeAsset,
}

impl ActiveModelBehavior for ActiveModel {}
