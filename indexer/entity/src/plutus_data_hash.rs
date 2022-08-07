use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "PlutusDataHash")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub hash: Vec<u8>,
    #[sea_orm(column_type = "BigInteger")]
    pub first_tx: i64,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::FirstTx",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
    #[sea_orm(has_one = "super::plutus_data::Entity")]
    PlutusData,
}

impl ActiveModelBehavior for ActiveModel {}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}
