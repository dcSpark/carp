use dcspark_blockchain_source::cardano::Point;
use dcspark_blockchain_source::{EventObject, GetNextFrom};
use dcspark_core::{BlockId, SlotNumber};

#[derive(Debug)]
pub enum CardanoEventType {
    RollBack {
        block_slot: u64,
        block_hash: String,
    },
    Block {
        cbor_hex: String,
        epoch: Option<u64>,
        epoch_slot: Option<u64>,
        block_number: u64,
        block_hash: String,
        block_slot: u64,
    },
}

impl GetNextFrom for CardanoEventType {
    type From = Point;

    fn next_from(&self) -> Option<Self::From> {
        match &self {
            CardanoEventType::RollBack { .. } => None,
            CardanoEventType::Block {
                block_hash,
                block_slot,
                ..
            } => Some(Point::BlockHeader {
                slot_nb: SlotNumber::new(*block_slot),
                hash: BlockId::new(block_hash.clone()),
            }),
        }
    }
}

impl EventObject for CardanoEventType {
    fn is_blockchain_tip(&self) -> bool {
        false
    }
}
