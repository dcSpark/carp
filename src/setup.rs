use std::{fmt, ops::Deref, str::FromStr, sync::Arc, thread::JoinHandle};

use anyhow::anyhow;
use cardano_serialization_lib::chain_crypto::Sha3_256;
use minicbor::data::Tag;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2n, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use entity::{
    prelude::{
        AddressActiveModel, Block, BlockActiveModel, BlockColumn, TransactionActiveModel,
        TransactionOutputActiveModel,
    },
    sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, QuerySelect, Set},
};
use pallas::ledger::primitives::byron::Blake2b224;

use crate::types::{crc32, GenesisFile};

pub async fn get_latest_points(conn: &DatabaseConnection) -> anyhow::Result<Vec<PointArg>> {
    let points: Vec<PointArg> = Block::find()
        .order_by_desc(BlockColumn::Id)
        .limit(1)
        .all(conn)
        .await?
        .iter()
        .map(|block| PointArg(block.slot as u64, hex::encode(&block.hash)))
        .collect();

    Ok(points)
}

pub fn oura_bootstrap(
    points: Vec<PointArg>,
    network: &str,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    let intersect = if !points.is_empty() {
        Some(IntersectArg::Fallbacks(points))
    } else {
        Some(IntersectArg::Origin)
    };

    let magic = MagicArg::from_str(network).map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known));

    let mapper = mapper::Config {
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2n::Config {
        address: AddressArg(BearerKind::Tcp, socket),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
        min_depth: 0,
        intersect,
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let mut handles = Vec::new();

    let (source_handle, source_rx) = source_setup
        .bootstrap()
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap source"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}

const GENESIS_MAINNET: &str = include_str!("../genesis/mainnet.json");
const GENESIS_TESTNET: &str = include_str!("../genesis/testnet.json");

pub async fn insert_genesis(conn: &DatabaseConnection, network: &str) -> anyhow::Result<()> {
    let genesis_str = match network {
        "mainnet" => GENESIS_MAINNET,
        "testnet" => GENESIS_TESTNET,
        rest => {
            return Err(anyhow!(
                "{} is invalid. NETWORK must be either mainnet or testnet",
                rest
            ))
        }
    };

    let genesis: GenesisFile = serde_json::from_str(genesis_str)?;

    tracing::info!("Parsed Genesis File and Beginning Hydration");

    let block = BlockActiveModel {
        era: Set(0),
        hash: Set(vec![]),
        height: Set(0),
        epoch: Set(0),
        slot: Set(0),
        payload: Set(vec![]),
        ..Default::default()
    };

    let block = block.insert(conn).await?;

    let magic = MagicArg::from_str(network).map_err(|_| anyhow!("shouldn't fail"))?;

    for (avvm, _) in genesis.avvm_distr.iter() {
        let bytes = base64::decode_config(avvm, base64::URL_SAFE)?;

        let pubkey = PublicKey::from_slice(&bytes)?;

        let address = ExtendedAddr::new(
            AddrType::Redeem,
            SpendingData::RedeemASD(pubkey),
            Attributes::new_bootstrap_era(None, NetworkMagic::Magic(*magic as u32)),
        );

        let payload = minicbor::to_vec(address)?;

        let transaction = TransactionActiveModel {
            block_id: Set(block.id),
            hash: Set(payload.clone()),
            is_valid: Set(true),
            payload: Set(vec![]),
            tx_index: Set(0),
            ..Default::default()
        };

        let transaction = transaction.insert(conn).await?;

        let address = AddressActiveModel {
            payload: Set(payload),
            ..Default::default()
        };

        let address = address.insert(conn).await?;

        let tx_output = TransactionOutputActiveModel {
            address_id: Set(address.id),
            tx_id: Set(transaction.id),
            payload: Set(vec![]),
            output_index: Set(0),
            ..Default::default()
        };

        tx_output.save(conn).await?;
    }

    Ok(())
}

const PUBLICKEY_SIZE: usize = 32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct PublicKey([u8; PUBLICKEY_SIZE]);
impl PublicKey {
    pub fn from_bytes(bytes: [u8; PUBLICKEY_SIZE]) -> Self {
        PublicKey(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != PUBLICKEY_SIZE {
            return Err(anyhow!("invalide public key size {}", bytes.len()));
        }

        let mut buf = [0; PUBLICKEY_SIZE];

        buf[0..PUBLICKEY_SIZE].clone_from_slice(bytes);

        Ok(Self::from_bytes(buf))
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

impl minicbor::Encode for PublicKey {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(self.as_ref())?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for PublicKey {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        PublicKey::from_slice(d.bytes()?)
            .map_err(|_| minicbor::decode::Error::message("cbor error"))
    }
}

/// A valid cardano address deconstructed
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedAddr {
    pub addr: HashedSpendingData,
    pub attributes: Attributes,
    pub addr_type: AddrType,
}

impl ExtendedAddr {
    pub fn new(ty: AddrType, sd: SpendingData, attrs: Attributes) -> Self {
        ExtendedAddr {
            addr: HashedSpendingData::new(ty, &sd, &attrs),
            attributes: attrs,
            addr_type: ty,
        }
    }
}

impl minicbor::Encode for ExtendedAddr {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut se = minicbor::Encoder::new(Vec::new());

        se.encode((&self.addr, &self.attributes, &self.addr_type))
            .unwrap();

        let bytes = se.end().unwrap();

        let crc32 = crc32(bytes.as_ref());

        e.array(2)?;
        e.tag(Tag::Unassigned(24))?;
        e.bytes(bytes.as_ref())?;
        e.u64(crc32 as u64)?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for ExtendedAddr {
    fn decode(_d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        todo!()
    }
    // fn deserialize<R: BufRead>(reader: &mut Deserializer<R>) -> cbor_event::Result<Self> {
    //     let bytes = cbor::hs::util::raw_with_crc32(reader)?;
    //     let mut raw = Deserializer::from(std::io::Cursor::new(bytes));
    //     raw.tuple(3, "ExtendedAddr")?;
    //     let addr = cbor_event::de::Deserialize::deserialize(&mut raw)?;
    //     let attributes = cbor_event::de::Deserialize::deserialize(&mut raw)?;
    //     let addr_type = cbor_event::de::Deserialize::deserialize(&mut raw)?;

    //     Ok(ExtendedAddr {
    //         addr,
    //         addr_type,
    //         attributes,
    //     })
    // }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum AddrType {
    PubKey,
    Script,
    Redeem,
}

impl fmt::Display for AddrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddrType::PubKey => write!(f, "Public Key"),
            AddrType::Script => write!(f, "Script"),
            AddrType::Redeem => write!(f, "Redeem"),
        }
    }
}

// [TkListLen 1, TkInt (fromEnum t)]
impl minicbor::Encode for AddrType {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let uint = match self {
            AddrType::PubKey => 0,
            AddrType::Script => 1,
            AddrType::Redeem => 2,
        };

        e.u64(uint)?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for AddrType {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        match d.u64()? {
            0 => Ok(AddrType::PubKey),
            1 => Ok(AddrType::Script),
            2 => Ok(AddrType::Redeem),
            _ => Err(minicbor::decode::Error::message("invalid addr type")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "generic-serialization", derive(Serialize, Deserialize))]
pub enum NetworkMagic {
    NoMagic,
    Magic(u32), // FIXME: should by i32
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Attributes {
    pub derivation_path: Option<HDAddressPayload>,
    pub stake_distribution: StakeDistribution,
    pub network_magic: NetworkMagic,
}

impl Attributes {
    pub fn new_bootstrap_era(hdap: Option<HDAddressPayload>, network_magic: NetworkMagic) -> Self {
        Attributes {
            derivation_path: hdap,
            stake_distribution: StakeDistribution::BootstrapEraDistr,
            network_magic,
        }
    }
}

const ATTRIBUTE_NAME_TAG_STAKE: u64 = 0;
const ATTRIBUTE_NAME_TAG_DERIVATION: u64 = 1;
const ATTRIBUTE_NAME_TAG_NETWORK_MAGIC: u64 = 2;

impl minicbor::Encode for Attributes {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut len = 0;

        match self.stake_distribution {
            StakeDistribution::BootstrapEraDistr => (),
            StakeDistribution::SingleKeyDistr(_) => len += 1,
        }

        if self.derivation_path.is_some() {
            len += 1
        }

        if let NetworkMagic::Magic(_) = &self.network_magic {
            len += 1
        }

        e.map(len)?;

        match self.stake_distribution {
            StakeDistribution::BootstrapEraDistr => (),
            StakeDistribution::SingleKeyDistr(_) => {
                e.u64(ATTRIBUTE_NAME_TAG_STAKE)?;
                e.encode(self.stake_distribution)?;
            }
        }

        match self.derivation_path {
            None => (),
            Some(ref dp) => {
                e.u64(ATTRIBUTE_NAME_TAG_DERIVATION)?;
                e.encode(dp)?;
            }
        }

        match &self.network_magic {
            NetworkMagic::NoMagic => (),
            NetworkMagic::Magic(network_magic) => {
                e.u64(ATTRIBUTE_NAME_TAG_NETWORK_MAGIC)?;

                let mut se = minicbor::Encoder::new(Vec::new());

                network_magic.encode(&mut se).unwrap();

                e.bytes(se.end().unwrap().as_ref())?;
            }
        }

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for Attributes {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let len = d.map()?;

        let mut len = match len {
            None => {
                return Err(minicbor::decode::Error::message(
                    "invalid attributes received",
                ))
            }
            Some(n) => n,
        };

        let mut stake_distribution = StakeDistribution::BootstrapEraDistr;
        let mut derivation_path = None;
        let mut network_magic = NetworkMagic::NoMagic;

        while len > 0 {
            let key = d.u64()?;

            match key {
                ATTRIBUTE_NAME_TAG_STAKE => stake_distribution = d.decode()?,
                ATTRIBUTE_NAME_TAG_DERIVATION => derivation_path = Some(d.decode()?),
                ATTRIBUTE_NAME_TAG_NETWORK_MAGIC => {
                    // Yes, this is an integer encoded as CBOR encoded as Bytes in CBOR.
                    let bytes = d.bytes()?;

                    let mut de = minicbor::Decoder::new(bytes);

                    let n = de.u32()?;

                    network_magic = NetworkMagic::Magic(n);
                }
                _ => {
                    return Err(minicbor::decode::Error::message("invalid Attribute key"));
                }
            }
            len -= 1;
        }

        Ok(Attributes {
            derivation_path,
            stake_distribution,
            network_magic,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct StakeholderId(Blake2b224);
impl StakeholderId {
    pub fn as_hash_bytes(&self) -> &[u8; 28] {
        &self.0
    }
}

impl minicbor::Encode for StakeholderId {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(self.0.as_ref())?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for StakeholderId {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let mut buf = [0; 28];

        let bytes = d.bytes()?;

        buf[0..28].clone_from_slice(bytes);

        Ok(StakeholderId(Blake2b224::new(buf)))
    }
}

impl fmt::Display for StakeholderId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl AsRef<[u8]> for StakeholderId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

// Blake2b224
impl From<StakeholderId> for Blake2b224 {
    fn from(hash: StakeholderId) -> Self {
        hash.0
    }
}

impl From<[u8; 28]> for StakeholderId {
    fn from(hash: [u8; 28]) -> Self {
        StakeholderId(Blake2b224::from(hash))
    }
}

impl From<Blake2b224> for StakeholderId {
    fn from(hash: Blake2b224) -> Self {
        StakeholderId(hash)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum StakeDistribution {
    BootstrapEraDistr,
    SingleKeyDistr(StakeholderId),
}

const STAKE_DISTRIBUTION_TAG_BOOTSTRAP: u64 = 1;
const STAKE_DISTRIBUTION_TAG_SINGLEKEY: u64 = 0;

impl StakeDistribution {
    pub fn new_bootstrap_era() -> Self {
        StakeDistribution::BootstrapEraDistr
    }
    pub fn new_single_stakeholder(si: StakeholderId) -> Self {
        StakeDistribution::SingleKeyDistr(si)
    }
}

impl minicbor::Encode for StakeDistribution {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut se = minicbor::Encoder::new(Vec::new());

        match self {
            StakeDistribution::BootstrapEraDistr => {
                se.array(1).unwrap();

                se.u64(STAKE_DISTRIBUTION_TAG_BOOTSTRAP).unwrap();
            }
            StakeDistribution::SingleKeyDistr(ref si) => {
                se.array(2).unwrap();

                se.u64(STAKE_DISTRIBUTION_TAG_SINGLEKEY).unwrap();

                se.encode(si).unwrap();
            }
        };

        e.bytes(se.end().unwrap().as_ref())?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for StakeDistribution {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;

        let mut de = minicbor::Decoder::new(bytes);

        let len = de.array()?;

        if len != Some(1) || len != Some(2) {
            return Err(minicbor::decode::Error::message("wrong number of elements"));
        }

        let sum_type_idx = de.u64()?;

        match sum_type_idx {
            STAKE_DISTRIBUTION_TAG_BOOTSTRAP => Ok(StakeDistribution::new_bootstrap_era()),
            STAKE_DISTRIBUTION_TAG_SINGLEKEY => {
                Ok(StakeDistribution::new_single_stakeholder(de.decode()?))
            }
            _ => Err(minicbor::decode::Error::message("Unsupported idx")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct HashedSpendingData(Blake2b224);

impl HashedSpendingData {
    pub fn new(addr_type: AddrType, spending_data: &SpendingData, attrs: &Attributes) -> Self {
        // the reason for this unwrap is that we have to dynamically allocate 66 bytes
        // to serialize 64 bytes in cbor (2 bytes of cbor overhead).

        let buffer = minicbor::to_vec((&addr_type, spending_data, attrs))
            .expect("serialize the HashedSpendingData's digest data");

        let hash = Sha3_256::new(&buffer);

        let mut buf = [0; 28];

        buf[0..28].clone_from_slice(&hash.as_ref()[0..28]);

        HashedSpendingData(Blake2b224::new(buf))
    }

    pub fn as_hash_bytes(&self) -> &[u8; 28] {
        &self.0
    }
}

impl fmt::Display for HashedSpendingData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl minicbor::Encode for HashedSpendingData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(self.as_hash_bytes())?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for HashedSpendingData {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;

        let mut buf = [0; 28];

        buf[0..28].clone_from_slice(bytes);

        Ok(HashedSpendingData(Blake2b224::new(buf)))
    }
}

impl AsRef<[u8]> for HashedSpendingData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<HashedSpendingData> for Blake2b224 {
    fn from(hash: HashedSpendingData) -> Self {
        hash.0
    }
}

impl From<[u8; 28]> for HashedSpendingData {
    fn from(hash: [u8; 28]) -> Self {
        HashedSpendingData(Blake2b224::from(hash))
    }
}

impl From<Blake2b224> for HashedSpendingData {
    fn from(hash: Blake2b224) -> Self {
        HashedSpendingData(hash)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]

pub struct HDAddressPayload(Vec<u8>);

impl AsRef<[u8]> for HDAddressPayload {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl HDAddressPayload {
    pub fn from_vec(v: Vec<u8>) -> Self {
        HDAddressPayload(v)
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        HDAddressPayload::from_vec(bytes.to_vec())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl minicbor::Encode for HDAddressPayload {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let data = self.0.as_slice();

        let mut se = minicbor::Encoder::new(Vec::new());

        data.encode(&mut se).unwrap();

        e.bytes(se.end().unwrap().as_ref())?;

        Ok(())
    }
}

impl<'b> minicbor::Decode<'b> for HDAddressPayload {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let inner_cbor = d.bytes()?;

        let mut inner_cbor = minicbor::Decoder::new(inner_cbor);

        Ok(HDAddressPayload::from_bytes(inner_cbor.bytes()?))
    }
}

impl Deref for HDAddressPayload {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl fmt::Debug for HDAddressPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

// pub type Script = [u8; 32]; // TODO

// const SPENDING_DATA_TAG_PUBKEY: u64 = 0;
// const SPENDING_DATA_TAG_SCRIPT: u64 = 1; // TODO
const SPENDING_DATA_TAG_REDEEM: u64 = 2;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpendingData {
    // PubKeyASD(XPub),
    // ScriptASD(Script),
    RedeemASD(PublicKey), // UnknownASD... whatever...
}

impl minicbor::Encode for SpendingData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            SpendingData::RedeemASD(pubkey) => {
                e.array(2)?;
                e.u64(SPENDING_DATA_TAG_REDEEM)?;
                e.encode(pubkey)?;

                Ok(())
            }
        }
    }
}
