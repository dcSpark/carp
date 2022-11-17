use super::utils::common::{
    get_asset_amount, get_plutus_datum_for_output, get_sheley_payment_hash,
};
use crate::multiera::dex::common::{
    WR_V1_POOL_SCRIPT_HASH, WR_V1_POOL_FIXED_ADA,
    build_asset, handle_mean_price, reduce_ada_amount, Dex, DexType, QueuedMeanPrice,
    WingRidersV1
};
use pallas::ledger::primitives::alonzo;
use super::{multiera_address::MultieraAddressTask, utils::common::asset_from_pair};
use crate::config::EmptyConfig::EmptyConfig;

use pallas::ledger::{
  primitives::{alonzo::Certificate, Fragment},
  traverse::{MultiEraBlock, MultiEraCert, MultiEraOutput, MultiEraTx},
};
use crate::dsl::task_macro::*;

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

pub fn get_pool_output<'b>(tx: &'b MultiEraTx) -> Option<(MultiEraOutput<'b>, alonzo::PlutusData)> {
    // Note: there should be at most one pool output
    if let Some(output) = tx
        .outputs()
        .iter()
        .find(|o| get_sheley_payment_hash(o.address()).as_deref() == Some(WR_V1_POOL_SCRIPT_HASH))
    {
        // Remark: The datum that corresponds to the pool output's datum hash should be present
        // in tx.plutus_data()
        if let Some(datum) = get_plutus_datum_for_output(output, &tx.plutus_data()) {
            Some((output.clone(), datum))
        } else {
            None
        }
    } else {
        None
    }
}
