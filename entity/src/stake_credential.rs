use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "StakeCredential")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub credential: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::tx_credential::Entity")]
    TxCredential,
}

impl ActiveModelBehavior for ActiveModel {}
