use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "StakeCredential")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(unique)]
    pub credential: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::tx_credential::Entity")]
    TxCredential,
    #[sea_orm(has_many = "super::address_credential::Entity")]
    AddressCredential,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::address_credential::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AddressCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
