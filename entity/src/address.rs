use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Address")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub payload: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::address_credential::Entity")]
    AddressCredential,
    #[sea_orm(has_many = "super::transaction_output::Entity")]
    TransactionOutput,
}

impl ActiveModelBehavior for ActiveModel {}
