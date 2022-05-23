use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Transaction")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(unique)]
    pub hash: Vec<u8>,
    pub block_id: i32,
    pub tx_index: i32, // index in block
    pub payload: Vec<u8>,
    pub is_valid: bool,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::block::Entity",
        from = "Column::BlockId",
        to = "super::block::Column::Id"
    )]
    Block,
    #[sea_orm(has_many = "super::transaction_input::Entity")]
    TransactionInput,
    #[sea_orm(has_many = "super::transaction_output::Entity")]
    TransactionOutput,
    #[sea_orm(has_many = "super::tx_credential::Entity")]
    TxCredential,
    #[sea_orm(has_many = "super::address::Entity")]
    Address,
    #[sea_orm(has_many = "super::stake_credential::Entity")]
    StakeCredential,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::block::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Block.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
