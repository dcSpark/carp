use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub era: i32,
    pub hash: Vec<u8>,
    pub height: i32,
    pub epoch: i32,
    pub slot: i32,
    pub payload: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl ActiveModelBehavior for ActiveModel {}
