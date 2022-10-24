use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "NativeAsset")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub policy_id: Vec<u8>,
    pub asset_name: Vec<u8>,
    pub cip14_fingerprint: Vec<u8>,
    #[sea_orm(column_type = "BigInteger")]
    pub first_tx: i64,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::asset_mint::Entity")]
    AssetMint,
    #[sea_orm(has_many = "super::asset_mint::Entity")]
    Cip25Entry,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::FirstTx",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl ActiveModelBehavior for ActiveModel {}
