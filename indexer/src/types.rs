use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock<'a> {
    Byron(Box<byron::Block>),
    Compatible(Box<alonzo::Block<'a>>),
}
