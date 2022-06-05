use cardano_multiplatform_lib::{
    chain_crypto, chain_crypto::Ed25519, crypto::BlockHeaderHash, fees::LinearFee,
    genesis::byron::config::GenesisData, utils, utils::BigNum,
};
use entity::block::EraValue;
use std::collections::BTreeMap;
use std::time::SystemTime;
use tasks::dsl::database_task::BlockGlobalInfo;

pub type OwnedBlockInfo = (String, GenesisData, BlockGlobalInfo);

pub const GENESIS_HASH: [u8; 32] = [1; 32];

#[derive(Default)]
pub struct GenesisBlockBuilder {
    avvm_dist: BTreeMap<chain_crypto::PublicKey<Ed25519>, utils::Coin>,
}

impl GenesisBlockBuilder {
    pub fn build(&self) -> OwnedBlockInfo {
        let cbor = "".to_string();
        let block_type = GenesisData {
            genesis_prev: BlockHeaderHash::from(GENESIS_HASH),
            epoch_stability_depth: 0,
            start_time: SystemTime::UNIX_EPOCH,
            slot_duration: Default::default(),
            protocol_magic: Default::default(),
            fee_policy: LinearFee {
                constant: BigNum::from(0),
                coefficient: BigNum::from(0),
            },
            avvm_distr: self.avvm_dist.clone(),
            non_avvm_balances: Default::default(),
            boot_stakeholders: Default::default(),
        };
        let block_global_data = BlockGlobalInfo {
            era: EraValue::Byron,
            epoch: None,
            epoch_slot: None,
        };
        (cbor, block_type, block_global_data)
    }

    pub fn with_voucher(
        &mut self,
        pub_key: chain_crypto::PublicKey<Ed25519>,
        coin: utils::Coin,
    ) -> &mut Self {
        self.avvm_dist.insert(pub_key, coin);
        self
    }
}
