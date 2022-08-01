use cardano_multiplatform_lib::{
    byron::{AddressContent, ByronAddress, ProtocolMagic},
    chain_crypto::{self, Ed25519, KeyPair, PublicKey},
    crypto::{self, BlockHeaderHash},
    genesis::byron::parse::parse,
    genesis::byron::{config::GenesisData, parse::redeem_pubkey_to_txid},
    ledger::{
        alonzo::fees::LinearFee,
        common::value::{BigNum, Coin},
    },
};
use entity::{
    block::EraValue,
    prelude::{AddressModel, TransactionModel, TransactionOutputModel},
    sea_orm::{Database, DbConn},
};
use proptest::prop_compose;
use rand::{rngs::StdRng, CryptoRng, Rng, RngCore, SeedableRng};
use std::{
    collections::BTreeMap,
    fs,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tasks::{
    dsl::database_task::BlockGlobalInfo,
    utils::{blake2b256, TaskPerfAggregator},
};

pub type OwnedBlockInfo = (String, GenesisData, BlockGlobalInfo);

pub const GENESIS_HASH: [u8; 32] = [1; 32];

fn get_protocol_magic() -> ProtocolMagic {
    ProtocolMagic::new(10)
}

#[derive(Default)]
pub struct GenesisBlockBuilder {
    avvm_dist: BTreeMap<chain_crypto::PublicKey<Ed25519>, Coin>,
    non_avvm_balances: BTreeMap<ByronAddress, Coin>,
}

impl GenesisBlockBuilder {
    pub fn build(&self) -> OwnedBlockInfo {
        let cbor = "".to_string();
        let block_type = GenesisData {
            genesis_prev: BlockHeaderHash::from(GENESIS_HASH),
            epoch_stability_depth: 0,
            start_time: SystemTime::UNIX_EPOCH,
            slot_duration: Default::default(),
            protocol_magic: get_protocol_magic(),
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
        avvms: Vec<(chain_crypto::PublicKey<Ed25519>, Coin)>,
    ) -> &mut Self {
        for (pubkey, coin) in avvms {
            self.avvm_dist.insert(pubkey, coin);
        }
        self
    }

    pub fn with_non_avvms(&mut self, avvms: Vec<(ByronAddress, Coin)>) -> &mut Self {
        for (addr, coin) in avvms {
            self.non_avvm_balances.insert(addr, coin);
        }
        self
    }
}

pub async fn in_memory_db_conn() -> DbConn {
    Database::connect("sqlite::memory:").await.unwrap()
}

pub fn new_perf_aggregator() -> Arc<Mutex<TaskPerfAggregator>> {
    Default::default()
}

pub fn new_pubkey<R: RngCore + CryptoRng>(rng: &mut R) -> PublicKey<Ed25519> {
    let (_, pubkey) = KeyPair::<Ed25519>::generate(rng).into_keys();
    pubkey
}

pub fn new_address<R: RngCore + CryptoRng>(rng: &mut R) -> ByronAddress {
    AddressContent::new_redeem(
        &crypto::PublicKey::from_bytes(new_pubkey(rng).as_ref()).unwrap(),
        Some(get_protocol_magic()),
    )
    .to_address()
}

pub fn pubkey_as_byron(
    pubkey: &PublicKey<Ed25519>,
    protocol_magical: ProtocolMagic,
) -> ByronAddress {
    let address = AddressContent::new_redeem(
        &crypto::PublicKey::from_bytes(pubkey.as_ref()).unwrap(),
        Some(protocol_magical),
    );
    address.to_address()
}

pub fn pubkey_to_tx_hash(
    pubkey: &PublicKey<Ed25519>,
    protocol_magic: Option<ProtocolMagic>,
) -> Vec<u8> {
    let (tx_hash, _) = redeem_pubkey_to_txid(pubkey, protocol_magic);
    tx_hash.to_bytes().to_vec()
}

pub fn addr_to_tx_hash(addr: ByronAddress) -> Vec<u8> {
    blake2b256(&addr.to_bytes()).to_vec()
}

pub fn db_tx_to_tx_hash_and_byron(tx: &TransactionModel) -> (Vec<u8>, ByronAddress) {
    let tx_hash = tx.hash.clone();
    let byron = ByronAddress::from_bytes(tx.payload.clone()).unwrap();
    (tx_hash, byron)
}

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

const GENESIS_MAINNET: &str = "../genesis/mainnet-byron-genesis.json";
const GENESIS_TESTNET: &str = "../genesis/testnet-byron-genesis.json";

pub async fn mainnet_block_info() -> OwnedBlockInfo {
    genesis_block_info(GENESIS_MAINNET)
}

pub async fn testnet_block_info() -> OwnedBlockInfo {
    genesis_block_info(GENESIS_TESTNET)
}

fn genesis_block_info(path: &str) -> OwnedBlockInfo {
    let cbor = "".to_string();

    let file = fs::File::open(path).expect("Failed to open genesis file");
    let genesis_file: GenesisData = parse(file);

    let block_global_info = BlockGlobalInfo {
        era: EraValue::Byron,
        epoch: None,
        epoch_slot: None,
    };
    (cbor, genesis_file, block_global_info)
}

prop_compose! {
    pub fn deterministic_rng()(seed: [u8; 32]) -> StdRng {
        StdRng::from_seed(seed)
    }
}

prop_compose! {
    pub fn arbitrary_avvms()(
        mut rng in deterministic_rng(),
        size in 0..100,
    ) -> Vec<(PublicKey<Ed25519>, BigNum)>{
        let mut avvms = Vec::new();
        for _ in 0..size {
            let value: u64 = rng.gen();
            let coin = value.into();
            let addr = new_pubkey(&mut rng);
            avvms.push((addr, coin))
        }
        avvms
    }
}

prop_compose! {
    pub fn arbitrary_non_avvms()(
        mut rng in deterministic_rng(),
        size in 0..100,
    ) -> Vec<(ByronAddress, BigNum)> {
        let mut non_avvms = Vec::new();
        for _ in 0..size {
            let value: u64 = rng.gen();
            let coin = value.into();
            let addr = new_address(&mut rng);
            non_avvms.push((addr, coin))
        }
        non_avvms
    }
}

prop_compose! {
    pub fn arbitrary_block()(
        avvms in arbitrary_avvms(),
        non_avvms in arbitrary_non_avvms()
    ) -> OwnedBlockInfo {
        GenesisBlockBuilder::default()
            .with_avvms(avvms)
            .with_non_avvms(non_avvms)
            .build()
    }
}
