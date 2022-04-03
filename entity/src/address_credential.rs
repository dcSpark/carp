use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "AddressCredentialRelation")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub address_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub credential_id: i64,
    pub relation: i32,
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
        belongs_to = "super::stake_credential::Entity",
        from = "Column::CredentialId",
        to = "super::stake_credential::Column::Id"
    )]
    StakeCredential,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Address.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::stake_credential::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StakeCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
