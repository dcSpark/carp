use std::collections::BTreeSet;

use crate::config::ReadonlyConfig::ReadonlyConfig;
use crate::types::AddressCredentialRelationValue;
use entity::sea_orm::Condition;
use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};

use crate::dsl::task_macro::*;

use super::{
    multiera_stake_credentials::MultieraStakeCredentialTask,
};

use crate::config::AddressConfig;

carp_task! {
  name MultiEraProjectedNftTask;
  configuration AddressConfig;
  doc "Parses projected NFT contract data";
  era multiera;
  dependencies [MultieraStakeCredentialTask];
  read [multiera_txs, multiera_stake_credential];
  write [];
  should_add_task |block, _properties| {
    !block.1.is_empty()
  };
  execute |previous_data, task| handle_projeced_nft(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_stake_credential,
      task.config.address
  );
  merge_result |previous_data, _result| {
  };
}

async fn handle_projected_nft(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, MultiEraBlock<'_>, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
    multiera_stake_credential: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    address: String,
) -> Result<(), DbErr> {
    Ok(())
}
