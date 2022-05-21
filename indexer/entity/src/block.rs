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
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
    #[sea_orm(has_many = "super::tx_credential::Entity")]
    TxCredentialRelation,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EraValue {
    Byron,
    Shelley,
    Allegra,
    Mary,
    Alonzo,
}

impl From<EraValue> for i32 {
    fn from(item: EraValue) -> Self {
        match item {
            EraValue::Byron => 0,
            EraValue::Shelley => 1,
            EraValue::Allegra => 2,
            EraValue::Mary => 3,
            EraValue::Alonzo => 4,
        }
    }
}
