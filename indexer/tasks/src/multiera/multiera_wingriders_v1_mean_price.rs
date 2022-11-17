use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_shelley_payment_hash,
};
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::config::EmptyConfig::EmptyConfig;
use crate::multiera::dex::common::{handle_mean_price, DexType};
use pallas::ledger::primitives::alonzo;

use crate::dsl::task_macro::*;
use pallas::ledger::{
    primitives::{alonzo::Certificate, Fragment},
    traverse::{MultiEraBlock, MultiEraCert, MultiEraOutput, MultiEraTx},
};

carp_task! {
  name MultieraWingRidersV1MeanPriceTask;
  configuration EmptyConfig;
  doc "Adds WingRiders V1 mean price updates to the database";
  era multiera;
  dependencies [MultieraAddressTask];
  read [multiera_txs, multiera_addresses];
  write [];
  should_add_task |block, _properties| {
    block.1.txs().iter().any(|tx| tx.outputs().len() > 0)
  };
  execute |previous_data, task| handle_mean_price(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
      &previous_data.multiera_addresses,
      DexType::WingRidersV1,
  );
  merge_result |previous_data, _result| {
  };
}
