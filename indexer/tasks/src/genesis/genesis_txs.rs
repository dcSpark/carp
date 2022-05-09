extern crate shred;

use std::sync::{Arc, Mutex};

use cardano_multiplatform_lib::{
    address::ByronAddress,
    genesis::byron::{config::GenesisData, parse::redeem_pubkey_to_txid},
    utils::Value,
};
use entity::{
    prelude::*,
    sea_orm::{DatabaseTransaction, DbErr, EntityTrait, Set},
};
use nameof::name_of_type;
use shred::{DispatcherBuilder, ResourceId, System, SystemData, World, Write};

use crate::{
    database_task::{
        BlockInfo, DatabaseTaskMeta, GenesisTaskRegistryEntry, TaskBuilder, TaskRegistryEntry,
    },
    utils::{blake2b256, TaskPerfAggregator},
};
use entity::sea_orm::Iterable;
use futures::future::try_join;

#[derive(SystemData)]
pub struct Data<'a> {
    genesis_txs: Write<'a, Vec<TransactionModel>>,
    genesis_addresses: Write<'a, Vec<AddressModel>>,
    genesis_outputs: Write<'a, Vec<TransactionOutputModel>>,
}

pub struct GenesisTransactionTask<'a> {
    pub db_tx: &'a DatabaseTransaction,
    pub block: BlockInfo<'a, GenesisData>,
    pub handle: &'a tokio::runtime::Handle,
    pub perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
}

impl<'a> DatabaseTaskMeta<'a, GenesisData> for GenesisTransactionTask<'a> {
    const TASK_NAME: &'static str = name_of_type!(GenesisTransactionTask);
    const DEPENDENCIES: &'static [&'static str] = &[];

    fn new(
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, GenesisData>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
    ) -> Self {
        Self {
            db_tx,
            block,
            handle,
            perf_aggregator,
        }
    }
}

struct GenesisTransactionTaskBuilder;
impl<'a> TaskBuilder<'a, GenesisData> for GenesisTransactionTaskBuilder {
    fn get_name(&self) -> &'static str {
        GenesisTransactionTask::TASK_NAME
    }
    fn get_dependencies(&self) -> &'static [&'static str] {
        GenesisTransactionTask::DEPENDENCIES
    }

    fn add_task<'c>(
        &self,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'c>,
        db_tx: &'a DatabaseTransaction,
        block: BlockInfo<'a, GenesisData>,
        handle: &'a tokio::runtime::Handle,
        perf_aggregator: Arc<Mutex<TaskPerfAggregator>>,
        _properties: &ini::Properties,
    ) {
        let task = GenesisTransactionTask::new(db_tx, block, handle, perf_aggregator);
        dispatcher_builder.add(task, self.get_name(), self.get_dependencies());
    }
}

inventory::submit! {
    TaskRegistryEntry::Genesis(GenesisTaskRegistryEntry { builder: &GenesisTransactionTaskBuilder })
}

impl<'a> System<'a> for GenesisTransactionTask<'_> {
    type SystemData = Data<'a>;

    fn run(&mut self, mut bundle: Data<'a>) {
        let time_counter = std::time::Instant::now();

        let result = self
            .handle
            .block_on(handle_txs(self.db_tx, self.block))
            .unwrap();
        *bundle.genesis_txs = result.0;
        *bundle.genesis_addresses = result.1;
        *bundle.genesis_outputs = result.2;

        self.perf_aggregator
            .lock()
            .unwrap()
            .update(Self::TASK_NAME, time_counter.elapsed());
    }
}

async fn handle_txs(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, GenesisData>,
) -> Result<
    (
        Vec<TransactionModel>,
        Vec<AddressModel>,
        Vec<TransactionOutputModel>,
    ),
    DbErr,
> {
    // note: avvm added before non-avvm
    // https://github.com/input-output-hk/cardano-ledger/blob/ac51494e151af0ad99b937a787458ce71db0aaea/eras/byron/ledger/impl/src/Cardano/Chain/UTxO/GenesisUTxO.hs#L21

    let mut transactions: Vec<TransactionActiveModel> = vec![];
    // note: genesis file is a JSON structure, so there shouldn't be duplicate addresses
    // even across avvm and non-avvm it should be unique, otherwise two txs with the same tx hash would exist
    let mut addresses: Vec<AddressActiveModel> = vec![];
    let mut outputs: Vec<cardano_multiplatform_lib::TransactionOutput> = vec![];

    for (pub_key, amount) in block.1.avvm_distr.iter() {
        let (tx_hash, extended_addr) = redeem_pubkey_to_txid(pub_key, Some(block.1.protocol_magic));
        let byron_addr =
            ByronAddress::from_bytes(extended_addr.to_address().as_ref().to_vec()).unwrap();

        transactions.push(TransactionActiveModel {
            block_id: Set(block.2.id),
            hash: Set(tx_hash.to_bytes().to_vec()),
            is_valid: Set(true),
            payload: Set(byron_addr.to_bytes()),
            // note: strictly speaking, genesis txs are unordered so there is no defined index
            tx_index: Set(transactions.len() as i32),
            ..Default::default()
        });

        addresses.push(AddressActiveModel {
            payload: Set(byron_addr.to_bytes()),
            ..Default::default()
        });

        // TODO: this is actually wrong. CML uses the Shelley format, but this should be the Byron format
        outputs.push(cardano_multiplatform_lib::TransactionOutput::new(
            &byron_addr.to_address(),
            &Value::new(amount),
        ));
    }

    // note: empty on mainnet
    for (addr, amount) in block.1.non_avvm_balances.iter() {
        let byron_addr = ByronAddress::from_bytes(addr.as_ref().to_vec()).unwrap();

        let tx_hash = blake2b256(addr.as_ref());

        // println!("{}", amount.to_str());
        // println!("{}", byron_addr.to_base58());
        // println!("{}", hex::encode(tx_hash));

        transactions.push(TransactionActiveModel {
            block_id: Set(block.2.id),
            hash: Set(tx_hash.to_vec()),
            is_valid: Set(true),
            payload: Set(byron_addr.to_bytes()),
            // note: strictly speaking, genesis txs are unordered so there is no defined index
            tx_index: Set(transactions.len() as i32),
            ..Default::default()
        });

        addresses.push(AddressActiveModel {
            payload: Set(byron_addr.to_bytes()),
            ..Default::default()
        });

        // TODO: this is actually wrong. CML uses the Shelley format, but this should be the Byron format
        outputs.push(cardano_multiplatform_lib::TransactionOutput::new(
            &byron_addr.to_address(),
            &Value::new(amount),
        ));
    }

    let (inserted_txs, inserted_addresses) = try_join(
        bulk_insert_txs(db_tx, &transactions),
        Address::insert_many(addresses).exec_many_with_returning(db_tx),
    )
    .await?;

    let outputs_to_add = inserted_txs
        .iter()
        .zip(&inserted_addresses)
        .enumerate()
        .map(|(i, (tx, addr))| TransactionOutputActiveModel {
            address_id: Set(addr.id),
            tx_id: Set(tx.id),
            payload: Set(outputs[i].to_bytes()),
            // recall: genesis txs are hashes of addresses
            // so all txs have a single output
            output_index: Set(0),
            ..Default::default()
        });
    let inserted_outputs = TransactionOutput::insert_many(outputs_to_add)
        .exec_many_with_returning(db_tx)
        .await?;

    Ok((inserted_txs, inserted_addresses, inserted_outputs))
}

// https://github.com/SeaQL/sea-orm/issues/691
async fn bulk_insert_txs(
    txn: &DatabaseTransaction,
    transactions: &[TransactionActiveModel],
) -> Result<Vec<TransactionModel>, DbErr> {
    let mut result: Vec<TransactionModel> = vec![];
    for chunk in transactions
        .chunks((u16::MAX / <Transaction as EntityTrait>::Column::iter().count() as u16) as usize)
    {
        result.extend(
            Transaction::insert_many(chunk.to_vec())
                .exec_many_with_returning(txn)
                .await?,
        );
    }
    Ok(result)
}
