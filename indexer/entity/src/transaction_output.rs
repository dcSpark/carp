use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "TransactionOutput")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub payload: Vec<u8>,
    #[sea_orm(column_type = "BigInteger")]
    pub address_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub tx_id: i64,
    pub output_index: i32, // index inside transaction
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::address::Entity",
        from = "Column::AddressId",
        to = "super::address::Column::Id"
    )]
    Address,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
    #[sea_orm(has_one = "super::transaction_input::Entity")]
    TransactionInput,
    #[sea_orm(has_many = "super::transaction_reference_input::Entity")]
    TransactionReferenceInput,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Address.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
