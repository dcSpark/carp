use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "TransactionMetadata")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub tx_id: i64,
    #[sea_orm(primary_key)]
    pub label: Vec<u8>, // little-endian u64 ([u8; 8]) (https://github.com/launchbadge/sqlx/issues/1374)
    pub payload: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl ActiveModelBehavior for ActiveModel {}
