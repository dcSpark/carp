use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block>),
}
