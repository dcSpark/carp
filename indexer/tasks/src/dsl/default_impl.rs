use pallas::ledger::primitives::{alonzo, byron};

pub fn has_transaction_byron(block: &byron::Block) -> bool {
    match block {
        byron::Block::MainBlock(main_block) => main_block.body.tx_payload.len() > 0,
        _ => false,
    }
}

pub fn has_transaction_multiera(block: &alonzo::Block) -> bool {
    block.transaction_bodies.len() > 0
}
