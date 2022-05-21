use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Cip25Entry")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub metadata_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub asset_id: i64,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction_metadata::Entity",
        from = "Column::MetadataId",
        to = "super::transaction_metadata::Column::Id"
    )]
    TransactionMetadata,
    #[sea_orm(
        belongs_to = "super::native_asset::Entity",
        from = "Column::AssetId",
        to = "super::native_asset::Column::Id"
    )]
    NativeAsset,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransactionMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
