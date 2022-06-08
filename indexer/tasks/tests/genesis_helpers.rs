use cardano_multiplatform_lib::{
    address::ByronAddress,
    chain_crypto::{self, Ed25519, KeyPair, PublicKey},
    crypto::BlockHeaderHash,
    fees::LinearFee,
    genesis::{
        byron::config::GenesisData, byron::config::ProtocolMagic,
        byron::parse::redeem_pubkey_to_txid,
    },
    legacy_address,
    legacy_address::ExtendedAddr,
    utils::{self, BigNum},
};
use entity::prelude::AddressModel;
use entity::{
    block::EraValue,
    prelude::{TransactionModel, TransactionOutputModel},
    sea_orm::{Database, DbConn},
};
use rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tasks::{
    dsl::database_task::BlockGlobalInfo,
    utils::{blake2b256, TaskPerfAggregator},
};

pub type OwnedBlockInfo = (String, GenesisData, BlockGlobalInfo);

pub const GENESIS_HASH: [u8; 32] = [1; 32];
pub const PROTOCOL_MAGIC: ProtocolMagic = ProtocolMagic(10);

#[derive(Default)]
pub struct GenesisBlockBuilder {
    avvm_dist: BTreeMap<chain_crypto::PublicKey<Ed25519>, utils::Coin>,
    non_avvm_balances: BTreeMap<legacy_address::Addr, utils::Coin>,
}

impl GenesisBlockBuilder {
    pub fn build(&self) -> OwnedBlockInfo {
        let cbor = "".to_string();
        let block_type = GenesisData {
            genesis_prev: BlockHeaderHash::from(GENESIS_HASH),
            epoch_stability_depth: 0,
            start_time: SystemTime::UNIX_EPOCH,
            slot_duration: Default::default(),
            protocol_magic: PROTOCOL_MAGIC,
            fee_policy: LinearFee {
                constant: BigNum::from(0),
                coefficient: BigNum::from(0),
            },
            avvm_distr: self.avvm_dist.clone(),
            non_avvm_balances: self.non_avvm_balances.clone(),
            boot_stakeholders: Default::default(),
        };
        let block_global_data = BlockGlobalInfo {
            era: EraValue::Byron,
            epoch: None,
            epoch_slot: None,
        };
        (cbor, block_type, block_global_data)
    }

    pub fn with_avvms(
        &mut self,
        avvms: Vec<(chain_crypto::PublicKey<Ed25519>, utils::Coin)>,
    ) -> &mut Self {
        for (pubkey, coin) in avvms {
            self.avvm_dist.insert(pubkey, coin);
        }
        self
    }

    pub fn with_non_avvms(&mut self, avvms: Vec<(legacy_address::Addr, utils::Coin)>) -> &mut Self {
        for (addr, coin) in avvms {
            self.non_avvm_balances.insert(addr, coin);
        }
        self
    }
}

const RNG_SEED: [u8; 32] = [6; 32];

pub async fn in_memory_db_conn() -> DbConn {
    Database::connect("sqlite::memory:").await.unwrap()
}

pub fn new_perf_aggregator() -> Arc<Mutex<TaskPerfAggregator>> {
    Default::default()
}

pub fn new_rng() -> StdRng {
    StdRng::from_seed(RNG_SEED)
}

pub fn new_pubkey<R: RngCore + CryptoRng>(rng: &mut R) -> PublicKey<Ed25519> {
    let (_, pubkey) = KeyPair::<Ed25519>::generate(rng).into_keys();
    pubkey
}

pub fn new_address<R: RngCore + CryptoRng>(rng: &mut R) -> legacy_address::Addr {
    ExtendedAddr::new_redeem(&new_pubkey(rng), Some(PROTOCOL_MAGIC)).into()
}

pub fn some_block() -> OwnedBlockInfo {
    let mut rng = new_rng();
    let pubkey1 = new_pubkey(&mut rng);
    let pubkey2 = new_pubkey(&mut rng);
    let pubkey3 = new_pubkey(&mut rng);
    let coin1 = BigNum::from(100);
    let coin2 = BigNum::from(200);
    let coin3 = BigNum::from(50);

    let avvms = vec![(pubkey1, coin1), (pubkey2, coin2), (pubkey3, coin3)];

    let address1 = new_address(&mut rng);
    let address2 = new_address(&mut rng);
    let address3 = new_address(&mut rng);
    let coin1 = BigNum::from(444);
    let coin2 = BigNum::from(233);
    let coin3 = BigNum::from(11);

    let non_avvms = vec![(address1, coin1), (address2, coin2), (address3, coin3)];

    GenesisBlockBuilder::default()
        .with_avvms(avvms)
        .with_non_avvms(non_avvms)
        .build()
}

pub fn pubkey_as_byron(
    pubkey: &PublicKey<Ed25519>,
    protocol_magical: ProtocolMagic,
) -> ByronAddress {
    let address = ExtendedAddr::new_redeem(pubkey, Some(protocol_magical));
    ByronAddress::from_bytes(address.to_address().as_ref().to_vec()).unwrap()
}

pub fn pubkey_to_tx_hash(
    pubkey: &PublicKey<Ed25519>,
    protocol_magic: Option<ProtocolMagic>,
) -> Vec<u8> {
    let (tx_hash, _) = redeem_pubkey_to_txid(pubkey, protocol_magic);
    tx_hash.to_bytes().to_vec()
}

pub fn addr_as_byron(addr: legacy_address::Addr) -> ByronAddress {
    ByronAddress::from_bytes(addr.as_ref().to_vec()).unwrap()
}

pub fn addr_to_tx_hash(addr: legacy_address::Addr) -> Vec<u8> {
    blake2b256(addr.as_ref()).to_vec()
}

pub fn db_tx_to_enumerated_tx_hash(tx: &TransactionModel) -> Vec<u8> {
    tx.hash.clone()
}

// Is there a better way of doing this. Going from ExtendedAddr -> Pubkey seems... involved
pub fn db_output_as_byron_and_coin(output: &TransactionOutputModel) -> (ByronAddress, BigNum) {
    let payload = output.payload.clone();
    let cml_output = cardano_multiplatform_lib::TransactionOutput::from_bytes(payload).unwrap();
    let coin = cml_output.amount().coin();
    let address = cml_output.address();
    let byron_address = ByronAddress::from_address(&address).unwrap();
    (byron_address, coin)
}

pub fn db_address_as_byron(output: &AddressModel) -> ByronAddress {
    let payload = output.payload.clone();
    let byron_address = ByronAddress::from_bytes(payload).unwrap();
    byron_address
}
