use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "BlockMinter")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub key: Vec<u8>,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::block::Entity",
        from = "Column::Id",
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

impl ActiveModelBehavior for ActiveModel {}
