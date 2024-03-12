use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub era: i32,
    pub hash: Vec<u8>,
    pub height: i32,
    pub epoch: i32,
    pub slot: i32,
    pub payload: Option<Vec<u8>>,
    pub tx_count: i32,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EraValue {
    Byron,
    Shelley,
    Allegra,
    Mary,
    Alonzo,
    Babbage,
    Conway,
}

impl From<EraValue> for i32 {
    fn from(item: EraValue) -> Self {
        match item {
            EraValue::Byron => 0,
            EraValue::Shelley => 1,
            EraValue::Allegra => 2,
            EraValue::Mary => 3,
            EraValue::Alonzo => 4,
            EraValue::Babbage => 5,
            EraValue::Conway => 6,
        }
    }
}

impl TryFrom<i32> for EraValue {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EraValue::Byron),
            1 => Ok(EraValue::Shelley),
            2 => Ok(EraValue::Allegra),
            3 => Ok(EraValue::Mary),
            4 => Ok(EraValue::Alonzo),
            5 => Ok(EraValue::Babbage),
            6 => Ok(EraValue::Conway),
            _ => Err(()),
        }
    }
}
