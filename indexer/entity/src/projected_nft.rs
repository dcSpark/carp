use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "ProjectedNFT")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub utxo_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub tx_id: i64,
    pub operation: i32, // lock / unlock / claim
    // address_id here is useful for fast pagination without joining w/ txoutput table
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub asset_id: Option<i64>,
    pub amount: u64,
    pub plutus_datum: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::transaction_output::Entity",
    from = "Column::UtxoId",
    to = "super::transaction_output::Column::Id"
    )]
    TransactionOutput,
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
    Asset,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::native_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Asset.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction_output::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransactionOutput.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
