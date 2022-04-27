use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "TxCredentialRelation")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub credential_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub tx_id: i64,
    #[sea_orm(primary_key)]
    pub relation: i32,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stake_credential::Entity",
        from = "Column::CredentialId",
        to = "super::stake_credential::Column::Id"
    )]
    StakeCredential,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::stake_credential::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StakeCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
