extern crate shred;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::{
    codec::utils::MaybeIndefArray,
    ledger::primitives::{
        byron::{self, TxOut},
        Fragment,
    },
};
use shred::{Read, ResourceId, System, SystemData, World, Write};
use std::collections::BTreeMap;

use crate::era_common::get_truncated_address;

#[derive(SystemData)]
pub struct Data<'a> {
    byron_txs: Read<'a, Vec<TransactionModel>>,
    outputs: Write<'a, Vec<TransactionOutputModel>>,
}

pub struct ByronOutputTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: (&'a byron::Block, &'a BlockModel),
    pub handle: &'a tokio::runtime::Handle,
}

impl<'a> System<'a> for ByronOutputTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let result = self
            .handle
            .block_on(handle_outputs(
                self.db_tx,
                self.block,
                bundle.byron_txs.as_slice(),
            ))
            .unwrap();
        *bundle.outputs = result;
    }
}

async fn handle_outputs(
    db_tx: &DatabaseTransaction,
    block: (&byron::Block, &BlockModel),
    byron_txs: &[TransactionModel],
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    match &block.0 {
        // Byron era had Epoch-boundary blocks for calculating stake distribution changes
        // they don't contain any txs, so we can just ignore them
        byron::Block::EbBlock(_) => Ok(vec![]),
        byron::Block::MainBlock(main_block) => {
            let tx_outputs: Vec<_> = main_block
                .body
                .tx_payload
                .iter()
                .map(|payload| &payload.transaction.outputs)
                .zip(byron_txs)
                .collect();

            if tx_outputs.is_empty() {
                return Ok(vec![]);
            }
            // insert addresses
            let (_, address_map) = crate::era_common::insert_addresses(
                &tx_outputs
                    .iter()
                    .flat_map(|pair| pair.0.iter())
                    .map(|output| output.address.encode_fragment().unwrap())
                    .collect(),
                db_tx,
            )
            .await?;

            // note: outputs have to be added before inputs
            Ok(insert_byron_outputs(db_tx, &address_map, &tx_outputs).await?)
        }
    }
}

async fn insert_byron_outputs(
    txn: &DatabaseTransaction,
    address_map: &BTreeMap<Vec<u8>, AddressModel>,
    outputs: &[(&MaybeIndefArray<TxOut>, &TransactionModel)],
) -> Result<Vec<TransactionOutputModel>, DbErr> {
    let result = TransactionOutput::insert_many(
        outputs
            .iter()
            .flat_map(|pair| pair.0.iter().enumerate().zip(std::iter::repeat(pair.1)))
            .map(
                |((output_index, output), tx_id)| TransactionOutputActiveModel {
                    payload: Set(output.encode_fragment().unwrap()),
                    address_id: Set(address_map
                        .get(get_truncated_address(
                            &output.address.encode_fragment().unwrap(),
                        ))
                        .unwrap()
                        .id),
                    tx_id: Set(tx_id.id),
                    output_index: Set(output_index as i32),
                    ..Default::default()
                },
            ),
    )
    .exec_many_with_returning(txn)
    .await?;

    Ok(result)
}
