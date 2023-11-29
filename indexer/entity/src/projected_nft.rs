use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "ProjectedNFT")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub owner_address: Vec<u8>,
    pub previous_utxo_tx_hash: Vec<u8>,
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub previous_utxo_tx_output_index: Option<i64>,
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub hololocker_utxo_id: Option<i64>,
    #[sea_orm(column_type = "BigInteger")]
    pub tx_id: i64,
    pub asset: String,
    #[sea_orm(column_type = "BigInteger")]
    pub amount: i64,
    pub operation: i32, // lock / unlock / claim
    pub plutus_datum: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction_output::Entity",
        from = "Column::HololockerUtxoId",
        to = "super::transaction_output::Column::Id"
    )]
    TransactionOutput,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
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
