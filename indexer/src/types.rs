use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}

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
