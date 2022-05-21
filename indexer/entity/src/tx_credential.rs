use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "TxCredentialRelation")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub credential_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub tx_id: i64,
    /// Note: we add block to speed up pagination significantly
    /// To understand why, suppose we paginate between ((block, tx), block]
    /// If we don't have this field, we need to
    /// 1. Find all TxCredentialRelation related to the payload
    /// 2. Join all the credentials with all the Transactions that are within the block range
    /// Step (1) is non-trivial for some addresses which appear in have >1,000,000 txs
    /// This field allows us to shrink step (1) to only relations within the block range we care about
    pub block_id: i32,
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
    #[sea_orm(
        belongs_to = "super::block::Entity",
        from = "Column::BlockId",
        to = "super::block::Column::Id"
    )]
    Block,
}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::block::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Block.def()
    }
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
