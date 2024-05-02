use crate::config::EmptyConfig::EmptyConfig;
use crate::{dsl::task_macro::*, multiera::multiera_txs::MultieraTransactionTask};
use cml_crypto::Serialize;
use entity::governance_votes::{ActiveModel, Model};
use sea_orm::{prelude::*, Set};

carp_task! {
  name MultieraGovernanceVotingTask;
  configuration EmptyConfig;
  doc "";
  era multiera;
  dependencies [MultieraTransactionTask];
  read [multiera_txs];
  write [];
  should_add_task |block, _properties| {
    block.1.transaction_bodies().iter().any(|x| x.voting_procedures().is_some())
  };
  execute |previous_data, task| handle(
      task.db_tx,
      task.block,
      &previous_data.multiera_txs,
  );
  merge_result |_previous_data, _result| {};
}

async fn handle(
    db_tx: &DatabaseTransaction,
    block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,
    multiera_txs: &[TransactionModel],
) -> Result<(), DbErr> {
    let mut queued_inserts = vec![];
    for (tx_body, cardano_transaction) in block.1.transaction_bodies().iter().zip(multiera_txs) {
        let voting_procedures = if let Some(voting_procedures) = tx_body.voting_procedures() {
            voting_procedures
        } else {
            continue;
        };

        for (voter, gov_action_id) in voting_procedures.iter() {
            for (gov_action_id, vote) in gov_action_id.iter() {
                queued_inserts.push(ActiveModel {
                    tx_id: Set(cardano_transaction.id),
                    voter: Set(voter.to_cbor_bytes()),
                    gov_action_id: Set(gov_action_id.to_cbor_bytes()),
                    vote: Set(vote.to_cbor_bytes()),
                    ..Default::default()
                })
            }
        }
    }

    if !queued_inserts.is_empty() {
        GovernanceVote::insert_many(queued_inserts.into_iter())
            .exec(db_tx)
            .await?;
    }

    Ok(())
}
