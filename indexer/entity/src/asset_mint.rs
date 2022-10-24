use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "AssetMint")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub tx_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub asset_id: i64,
    pub amount: i64, // i64 according to spec. Negative = burn tokens
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
    #[sea_orm(
        belongs_to = "super::native_asset::Entity",
        from = "Column::AssetId",
        to = "super::native_asset::Column::Id"
    )]
    NativeAsset,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::native_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NativeAsset.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
