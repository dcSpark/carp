use crate::perf_aggregator::PerfAggregator;
use pallas::ledger::primitives::{
    alonzo::{self, Certificate, TransactionBody, TransactionBodyComponent},
    Fragment,
};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use cardano_multiplatform_lib::{
    address::{BaseAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    utils::ScriptHashNamespace,
};
use oura::model::BlockRecord;

use crate::{
    relation_map::RelationMap,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
};
use entity::{
    prelude::*,
    sea_orm::{prelude::*, sea_query::OnConflict, ColumnTrait, DatabaseTransaction, Set},
};

struct VirtualTransaction<'a> {
    body: &'a TransactionBody,
    witness_set: &'a cardano_multiplatform_lib::TransactionWitnessSet,
    is_valid: bool,
    database_index: i64,
}
pub async fn process_multiera_block(
    perf_aggregator: &mut PerfAggregator,
    time_counter: &mut std::time::Instant,
    txn: &DatabaseTransaction,
    block_record: &BlockRecord,
    db_block: &BlockModel,
    alonzo_block: &alonzo::Block,
) -> Result<(), DbErr> {
    let joined_txs: Vec<(
        TransactionActiveModel,
        &TransactionBody,
        cardano_multiplatform_lib::TransactionWitnessSet,
        bool,
    )> = alonzo_block
        .deref()
        .transaction_bodies
        .iter()
        .zip(alonzo_block.transaction_witness_sets.iter())
        .enumerate()
        .map(|(idx, (tx_body, tx_witness_set))| {
            let body_payload = tx_body.encode_fragment().unwrap();
            let body = &cardano_multiplatform_lib::TransactionBody::from_bytes(body_payload)
                .map_err(|e| {
                    panic!(
                        "{:?}\nBlock cbor: {:?}\nTransaction body cbor: {:?}\nTx hash: {:?}\n",
                        e,
                        block_record.cbor_hex,
                        hex::encode(tx_body.encode_fragment().unwrap()),
                        hex::encode(tx_body.to_hash().to_vec())
                    )
                })
                .unwrap();

            let witness_set_payload = tx_witness_set.encode_fragment().unwrap();
            let witness_set =
                &cardano_multiplatform_lib::TransactionWitnessSet::from_bytes(witness_set_payload)
                    .map_err(|e| panic!("{:?}\nBlock cbor: {:?}", e, block_record.cbor_hex))
                    .unwrap();

            let aux_data = alonzo_block
                .auxiliary_data_set
                .iter()
                .find(|(index, _)| *index as usize == idx);

            let auxiliary_data = aux_data.map(|(_, a)| {
                let auxiliary_data_payload = a.encode_fragment().unwrap();

                cardano_multiplatform_lib::metadata::AuxiliaryData::from_bytes(
                    auxiliary_data_payload,
                )
                .map_err(|e| {
                    panic!(
                        "{:?}\n{:?}\n{:?}",
                        e,
                        hex::encode(a.encode_fragment().unwrap()),
                        cardano_multiplatform_lib::Block::from_bytes(
                            hex::decode(block_record.cbor_hex.clone().unwrap()).unwrap(),
                        )
                        .map(|block| block.to_json())
                        .map_err(|_err| block_record.cbor_hex.clone().unwrap()),
                    )
                })
                .unwrap()
            });

            let mut temp_tx =
                cardano_multiplatform_lib::Transaction::new(body, witness_set, auxiliary_data);

            let mut is_valid = true;

            if let Some(ref invalid_txs) = alonzo_block.invalid_transactions {
                is_valid = !invalid_txs.iter().any(|i| *i as usize == idx)
            }

            temp_tx.set_is_valid(is_valid);

            let transaction = TransactionActiveModel {
                hash: Set(tx_body.to_hash().to_vec()),
                block_id: Set(db_block.id),
                tx_index: Set(idx as i32),
                payload: Set(temp_tx.to_bytes()),
                is_valid: Set(is_valid),
                ..Default::default()
            };

            (transaction, tx_body, witness_set.clone(), is_valid)
        })
        .collect();

    if joined_txs.len() > 0 {
        let insertions = Transaction::insert_many(joined_txs.iter().map(|tx| tx.0.clone()))
            .exec_with_returning(true, txn)
            .await?
            .unwrap();
        perf_aggregator.transaction_insert += time_counter.elapsed();
        *time_counter = std::time::Instant::now();

        process_multiera_txs(
            perf_aggregator,
            time_counter,
            txn,
            block_record,
            &joined_txs
                .iter()
                .enumerate()
                .map(|(i, tx)| VirtualTransaction {
                    body: tx.1,
                    witness_set: &tx.2,
                    is_valid: tx.3,
                    database_index: insertions.id + (i as i64),
                })
                .collect(),
        )
        .await?;
    }

    Ok(())
}

async fn process_multiera_txs<'a>(
    perf_aggregator: &mut PerfAggregator,
    time_counter: &mut std::time::Instant,
    txn: &DatabaseTransaction,
    block_record: &BlockRecord,
    cardano_transactions: &Vec<VirtualTransaction<'a>>,
) -> Result<(), DbErr> {
    for cardano_transaction in cardano_transactions.iter() {
        let vkey_relation_map = Arc::new(Mutex::new(RelationMap::default()));
        insert_witness(
            vkey_relation_map.clone(),
            cardano_transaction.database_index,
            cardano_transaction.witness_set,
            txn,
        )
        .await?;

        perf_aggregator.witness_insert += time_counter.elapsed();
        *time_counter = std::time::Instant::now();

        for component in cardano_transaction.body.iter() {
            match component {
                TransactionBodyComponent::Certificates(certs) => {
                    for cert in certs.iter() {
                        insert_certificate(
                            vkey_relation_map.clone(),
                            cardano_transaction.database_index,
                            &cert,
                            txn,
                        )
                        .await?;
                    }
                    perf_aggregator.certificate_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                TransactionBodyComponent::Outputs(outputs) => {
                    for (idx, output) in outputs.iter().enumerate() {
                        insert_output(
                            vkey_relation_map.clone(),
                            &block_record,
                            txn,
                            cardano_transaction.database_index,
                            output,
                            idx,
                        )
                        .await?;
                    }
                    perf_aggregator.transaction_output_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                TransactionBodyComponent::Withdrawals(withdrawal_pairs) => {
                    for pair in withdrawal_pairs.deref() {
                        let reward_addr = RewardAddress::from_address(
                            &cardano_multiplatform_lib::address::Address::from_bytes(
                                pair.0.clone().into(),
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        let address = crate::era_common::insert_address(
                            &mut reward_addr.to_address().to_bytes(),
                            txn,
                        )
                        .await?;
                        insert_address_credential(
                            vkey_relation_map.clone(),
                            cardano_transaction.database_index,
                            &reward_addr.payment_cred().to_bytes(),
                            &address,
                            TxCredentialRelationValue::Withdrawal.into(),
                            AddressCredentialRelationValue::PaymentKey.into(),
                            txn,
                        )
                        .await?;
                    }
                    perf_aggregator.withdrawal_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                TransactionBodyComponent::RequiredSigners(key_hashes) => {
                    for &signer in key_hashes.iter() {
                        let owner_credential =
                            pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(
                                signer,
                            )
                            .encode_fragment()
                            .unwrap();
                        insert_stake_credential(
                            vkey_relation_map.clone(),
                            cardano_transaction.database_index,
                            &owner_credential,
                            txn,
                            TxCredentialRelationValue::RequiredSigner.into(),
                        )
                        .await?;
                    }
                    perf_aggregator.required_signer_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                _ => (),
            }
        }

        // note: inputs have to be added AFTER outputs
        for component in cardano_transaction.body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) if cardano_transaction.is_valid => {
                    crate::era_common::insert_inputs(
                        vkey_relation_map.clone(),
                        cardano_transaction.database_index,
                        &inputs,
                        txn,
                    )
                    .await?;
                    perf_aggregator.transaction_input_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                TransactionBodyComponent::Collateral(inputs) if !cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    crate::era_common::insert_inputs(
                        vkey_relation_map.clone(),
                        cardano_transaction.database_index,
                        &inputs,
                        txn,
                    )
                    .await?;
                    perf_aggregator.collateral_insert += time_counter.elapsed();
                    *time_counter = std::time::Instant::now();
                }
                _ => (),
            };
        }

        insert_tx_credentials(
            vkey_relation_map.clone(),
            cardano_transaction.database_index,
            txn,
        )
        .await?;
        perf_aggregator.tx_credential_relation += time_counter.elapsed();
        *time_counter = std::time::Instant::now();
    }

    Ok(())
}

async fn insert_output(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    block_record: &BlockRecord,
    txn: &DatabaseTransaction,
    tx_id: i64,
    output: &alonzo::TransactionOutput,
    idx: usize,
) -> Result<(), DbErr> {
    use cardano_multiplatform_lib::address::Address;

    let address = crate::era_common::insert_address(&mut output.address.to_vec(), txn).await?;

    let addr = Address::from_bytes(output.address.to_vec())
        .map_err(|e| panic!("{:?}{:?}", e, block_record.cbor_hex))
        .unwrap();

    let tx_relation = TxCredentialRelationValue::Output;
    let address_relation = AddressCredentialRelationValue::PaymentKey;

    if let Some(base_addr) = BaseAddress::from_address(&addr) {
        // Payment Key
        let payload = base_addr.payment_cred().to_bytes();

        insert_address_credential(
            vkey_relation_map.clone(),
            tx_id,
            &payload,
            &address,
            tx_relation.into(),
            address_relation.into(),
            txn,
        )
        .await?;

        // Stake Key
        let payload = base_addr.stake_cred().to_bytes();

        let address_relation = AddressCredentialRelationValue::StakeKey;

        insert_address_credential(
            vkey_relation_map.clone(),
            tx_id,
            &payload,
            &address,
            tx_relation.into(),
            address_relation.into(),
            txn,
        )
        .await?;
    } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
        let payload = ptr_addr.payment_cred().to_bytes();

        insert_address_credential(
            vkey_relation_map.clone(),
            tx_id,
            &payload,
            &address,
            tx_relation.into(),
            address_relation.into(),
            txn,
        )
        .await?;
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&addr) {
        let payload = enterprise_addr.payment_cred().to_bytes();

        insert_address_credential(
            vkey_relation_map.clone(),
            tx_id,
            &payload,
            &address,
            tx_relation.into(),
            address_relation.into(),
            txn,
        )
        .await?;
    } else if let Some(reward_addr) = RewardAddress::from_address(&addr) {
        let payload = reward_addr.payment_cred().to_bytes();
        insert_address_credential(
            vkey_relation_map.clone(),
            tx_id,
            &payload,
            &address,
            tx_relation.into(),
            address_relation.into(),
            txn,
        )
        .await?;
    };

    let tx_output = TransactionOutputActiveModel {
        payload: Set(output.encode_fragment().unwrap()),
        address_id: Set(address.id),
        tx_id: Set(tx_id),
        output_index: Set(idx as i32),
        ..Default::default()
    };

    tx_output.save(txn).await?;

    Ok(())
}

async fn insert_address_credential(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    payload: &Vec<u8>,
    address: &AddressModel,
    tx_relation: TxCredentialRelationValue,
    address_relation: i32, // TODO: type
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let stake_credential =
        insert_stake_credential(vkey_relation_map.clone(), tx_id, payload, txn, tx_relation)
            .await?;

    let address_credential = AddressCredentialActiveModel {
        credential_id: Set(stake_credential.id),
        address_id: Set(address.id),
        relation: Set(address_relation),
    };

    // As of April 15th, 2022, there are:
    // total txs = 37,713,207
    // total addresses = 3,239,919
    // which means for every 10txs, there is 1 new address
    // we still prefer to write(on conflict) instead of read-then-write because of Postgres MVCC
    let on_conflict = OnConflict::columns([
        AddressCredentialColumn::AddressId,
        AddressCredentialColumn::CredentialId,
        AddressCredentialColumn::Relation,
    ])
    .do_nothing()
    .to_owned();
    address_credential.insert_or(txn, &on_conflict).await?;

    Ok(())
}

async fn insert_tx_credentials(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let models = match vkey_relation_map.lock().unwrap().0.get(&tx_id) {
        Some(mapping) => mapping
            .values()
            .map(|val| TxCredentialActiveModel {
                credential_id: Set(val.credential_id),
                tx_id: Set(tx_id),
                relation: Set(val.relation),
            })
            .collect(),
        None => vec![],
    };

    // no entries to add if tx only had Byron addresses
    if models.len() > 0 {
        TxCredential::insert_many(models).exec(txn).await?;
    }
    Ok(())
}

async fn insert_certificate(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    cert: &Certificate,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    match cert {
        Certificate::StakeDelegation(credential, pool) => {
            let credential = credential.encode_fragment().unwrap();
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &credential,
                txn,
                TxCredentialRelationValue::StakeDelegation.into(),
            )
            .await?;
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &RelationMap::keyhash_to_pallas(
                    &cardano_multiplatform_lib::crypto::Ed25519KeyHash::from_bytes(pool.to_vec())
                        .unwrap(),
                )
                .to_vec(),
                txn,
                TxCredentialRelationValue::DelegationTarget.into(),
            )
            .await?;
        }
        Certificate::StakeRegistration(credential) => {
            let credential = credential.encode_fragment().unwrap();
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &credential,
                txn,
                TxCredentialRelationValue::StakeDelegation.into(),
            )
            .await?;
        }
        Certificate::StakeDeregistration(credential) => {
            let credential = credential.encode_fragment().unwrap();
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &credential,
                txn,
                TxCredentialRelationValue::StakeDeregistration.into(),
            )
            .await?;
        }
        Certificate::PoolRegistration {
            operator,
            pool_owners,
            reward_account,
            ..
        } => {
            let operator_credential =
                pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(operator.clone())
                    .encode_fragment()
                    .unwrap();
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &operator_credential,
                txn,
                TxCredentialRelationValue::PoolOperator.into(),
            )
            .await?;

            let reward_addr = RewardAddress::from_address(
                &cardano_multiplatform_lib::address::Address::from_bytes(reward_account.to_vec())
                    .unwrap(),
            )
            .unwrap();
            let address =
                crate::era_common::insert_address(&mut reward_addr.to_address().to_bytes(), txn)
                    .await?;
            insert_address_credential(
                vkey_relation_map.clone(),
                tx_id,
                &reward_addr.payment_cred().to_bytes(),
                &address,
                TxCredentialRelationValue::PoolReward.into(),
                AddressCredentialRelationValue::PaymentKey.into(),
                txn,
            )
            .await?;

            for &owner in pool_owners.iter() {
                let owner_credential =
                    pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(owner)
                        .encode_fragment()
                        .unwrap();
                insert_stake_credential(
                    vkey_relation_map.clone(),
                    tx_id,
                    &owner_credential,
                    txn,
                    TxCredentialRelationValue::PoolOperator.into(),
                )
                .await?;
            }
        }
        Certificate::PoolRetirement(key_hash, _) => {
            let operator_credential =
                pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(key_hash.clone())
                    .encode_fragment()
                    .unwrap();
            insert_stake_credential(
                vkey_relation_map.clone(),
                tx_id,
                &operator_credential,
                txn,
                TxCredentialRelationValue::PoolOperator.into(),
            )
            .await?;
        }
        Certificate::GenesisKeyDelegation(_, _, _) => {
            // genesis keys aren't stake credentials
        }
        Certificate::MoveInstantaneousRewardsCert(mir) => match &mir.target {
            pallas::ledger::primitives::alonzo::InstantaneousRewardTarget::StakeCredentials(
                credential_pairs,
            ) => {
                for pair in credential_pairs.deref() {
                    let credential = pair.0.encode_fragment().unwrap();
                    insert_stake_credential(
                        vkey_relation_map.clone(),
                        tx_id,
                        &credential,
                        txn,
                        TxCredentialRelationValue::MirRecipient.into(),
                    )
                    .await?;
                }
            }
            _ => {}
        },
    };

    Ok(())
}

async fn insert_witness(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    witness_set: &cardano_multiplatform_lib::TransactionWitnessSet,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    match witness_set.vkeys() {
        Some(vkeys) => {
            for i in 0..vkeys.len() {
                let vkey = vkeys.get(i);
                insert_stake_credential(
                    vkey_relation_map.clone(),
                    tx_id,
                    &RelationMap::keyhash_to_pallas(&vkey.vkey().public_key().hash()).to_vec(),
                    txn,
                    TxCredentialRelationValue::Witness,
                )
                .await?;
            }
        }
        _ => (),
    };

    match witness_set.native_scripts() {
        Some(scripts) => {
            for i in 0..scripts.len() {
                let script = scripts.get(i);
                insert_stake_credential(
                    vkey_relation_map.clone(),
                    tx_id,
                    &RelationMap::scripthash_to_pallas(
                        &script.hash(ScriptHashNamespace::NativeScript),
                    )
                    .to_vec(),
                    txn,
                    TxCredentialRelationValue::Witness,
                )
                .await?;
            }
        }
        _ => (),
    };

    match witness_set.plutus_scripts() {
        Some(scripts) => {
            for i in 0..scripts.len() {
                let script = scripts.get(i);
                insert_stake_credential(
                    vkey_relation_map.clone(),
                    tx_id,
                    // TODO: PlutusV2
                    &RelationMap::scripthash_to_pallas(&script.hash(ScriptHashNamespace::PlutusV1))
                        .to_vec(),
                    txn,
                    TxCredentialRelationValue::Witness,
                )
                .await?;
            }
        }
        _ => (),
    };

    Ok(())
}

async fn insert_stake_credential(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    credential: &Vec<u8>,
    txn: &DatabaseTransaction,
    tx_relation: TxCredentialRelationValue,
) -> Result<StakeCredentialModel, DbErr> {
    let staking_credential =
        fetch_stake_credential(vkey_relation_map.clone(), tx_id, credential, txn).await?;
    if let Some(stake_credential) = staking_credential {
        let mut vkey_relation_map = vkey_relation_map.lock().unwrap();
        vkey_relation_map.add_relation(
            tx_id,
            stake_credential.id,
            &stake_credential.credential,
            tx_relation,
        );

        Ok(stake_credential)
    } else {
        let stake_credential = StakeCredentialActiveModel {
            credential: Set(credential.clone()),
            ..Default::default()
        };

        let stake_credential = stake_credential.insert(txn).await?;

        let mut vkey_relation_map = vkey_relation_map.lock().unwrap();
        vkey_relation_map.add_relation(
            tx_id,
            stake_credential.id,
            &stake_credential.credential,
            tx_relation,
        );

        Ok(stake_credential)
    }
}

async fn fetch_stake_credential(
    vkey_relation_map: Arc<Mutex<RelationMap>>,
    tx_id: i64,
    credential: &Vec<u8>,
    txn: &DatabaseTransaction,
) -> Result<Option<StakeCredentialModel>, DbErr> {
    // note: this is purely an optimization because a lot of txs contain the same address multiple times
    // we may have already looked up this credential inside this transaction
    // so try and pull it from our local info before querying the DB
    let result: Result<Option<StakeCredentialModel>, DbErr> = (|| match &vkey_relation_map
        .lock()
        .unwrap()
        .for_transaction(tx_id)
        .entry(RelationMap::bytes_to_pallas(&credential))
    {
        std::collections::btree_map::Entry::Occupied(entry) => {
            let val = entry.get();
            Ok(Some(StakeCredentialModel {
                id: val.credential_id,
                credential: credential.clone(),
            }))
        }
        std::collections::btree_map::Entry::Vacant(_) => Ok(None),
    })();

    match result? {
        Some(val) => Ok(Some(val)),
        None => Ok(StakeCredential::find()
            .filter(StakeCredentialColumn::Credential.eq(credential.clone()))
            // note: we know this exists ("credential" is unique) and "all" is faster than "one" if we know the result exists
            .all(txn)
            .await?
            .first()
            .cloned()),
    }
}
