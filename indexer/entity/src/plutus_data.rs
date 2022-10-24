use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "PlutusData")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plutus_data_hash::Entity",
        from = "Column::Id",
        to = "super::plutus_data_hash::Column::Id"
    )]
    PlutusDataHash,
}

impl ActiveModelBehavior for ActiveModel {}

// TODO: figure out why this isn't automatically handle by the macros above
impl Related<super::plutus_data_hash::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlutusDataHash.def()
    }
}
