use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "DexSwap")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub tx_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub address_id: i64,
    pub dex: i32,
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub asset1_id: Option<i64>,
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub asset2_id: Option<i64>,
    #[sea_orm(column_type = "BigUnsigned")]
    pub amount1: u64,
    #[sea_orm(column_type = "BigUnsigned")]
    pub amount2: u64,
    pub direction: bool,
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
        belongs_to = "super::address::Entity",
        from = "Column::AddressId",
        to = "super::address::Column::Id"
    )]
    Address,
    #[sea_orm(
        belongs_to = "super::native_asset::Entity",
        from = "Column::Asset1Id",
        to = "super::native_asset::Column::Id"
    )]
    Asset1,
    #[sea_orm(
        belongs_to = "super::native_asset::Entity",
        from = "Column::Asset2Id",
        to = "super::native_asset::Column::Id"
    )]
    Asset2,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Address.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dex {
    WingRidersV1,
    SundaeSwapV1,
    MinSwapV1,
}

impl From<Dex> for i32 {
    fn from(item: Dex) -> Self {
        match item {
            Dex::WingRidersV1 => 0,
            Dex::SundaeSwapV1 => 1,
            Dex::MinSwapV1 => 2,
        }
    }
}
