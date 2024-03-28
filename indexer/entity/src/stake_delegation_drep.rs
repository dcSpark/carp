use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "StakeDelegationDrepCredentialRelation")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub stake_credential: i64,
    pub drep_credential: Option<Vec<u8>>,
    pub tx_id: i64,
    pub previous_drep_credential: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stake_credential::Entity",
        from = "Column::StakeCredential",
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

impl Related<super::stake_credential::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StakeCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
