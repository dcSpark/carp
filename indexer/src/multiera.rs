use crate::{era_common::get_truncated_address, perf_aggregator::PerfAggregator};
use pallas::ledger::primitives::{
    alonzo::{self, Certificate, TransactionBody, TransactionBodyComponent},
    Fragment,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use cardano_multiplatform_lib::{
    address::{BaseAddress, ByronAddress, EnterpriseAddress, PointerAddress, RewardAddress},
    utils::ScriptHashNamespace,
    RequiredSignersSet,
};
use oura::model::BlockRecord;

use crate::{
    relation_map::RelationMap,
    types::{AddressCredentialRelationValue, TxCredentialRelationValue},
};
use entity::{
    prelude::*,
    sea_orm::{
        prelude::*, ColumnTrait, Condition, DatabaseTransaction, JoinType, QuerySelect, Set,
    },
};

struct VirtualTransaction<'a> {
    body: &'a TransactionBody,
    witness_set: &'a cardano_multiplatform_lib::TransactionWitnessSet,
    is_valid: bool,
    database_index: i64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct QueuedAddressCredential {
    address: Vec<u8>,
    stake_credential: Vec<u8>, // pallas::crypto::hash::Hash<32>
    address_relation: AddressCredentialRelationValue,
}

struct QueuedOutput {
    // note: no need to use a map type
    // because the pair <tx_id, idx> should only ever be inserted once
    tx_id: i64,
    idx: usize,
    payload: Vec<u8>,
    address: Vec<u8>, // pallas::crypto::hash::Hash<32>
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
            .exec_many_with_returning(txn)
            .await?;
        perf_aggregator.transaction_insert += time_counter.elapsed();
        *time_counter = std::time::Instant::now();

        process_multiera_txs(
            perf_aggregator,
            time_counter,
            txn,
            &joined_txs
                .iter()
                .zip(&insertions)
                .map(|(tx, inserted)| VirtualTransaction {
                    body: tx.1,
                    witness_set: &tx.2,
                    is_valid: tx.3,
                    database_index: inserted.id,
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
    cardano_transactions: &Vec<VirtualTransaction<'a>>,
) -> Result<(), DbErr> {
    let mut vkey_relation_map = RelationMap::default();
    let mut queued_address_credential = BTreeSet::<QueuedAddressCredential>::default();
    let mut queued_address = BTreeSet::<Vec<u8>>::default();
    let mut queued_output = Vec::<QueuedOutput>::default();
    let mut queued_inputs = Vec::<(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )>::default();
    let mut queued_unused_inputs = Vec::<(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )>::default();

    for cardano_transaction in cardano_transactions.iter() {
        queue_witness(
            &mut vkey_relation_map,
            cardano_transaction.database_index,
            cardano_transaction.witness_set,
        );

        for component in cardano_transaction.body.iter() {
            match component {
                TransactionBodyComponent::Certificates(certs) => {
                    for cert in certs.iter() {
                        queue_certificate(
                            &mut vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            cardano_transaction.database_index,
                            &cert,
                        );
                    }
                }
                TransactionBodyComponent::Outputs(outputs) => {
                    for (idx, output) in outputs.iter().enumerate() {
                        queue_output(
                            &mut vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            &mut queued_output,
                            &cardano_transaction.body,
                            cardano_transaction.database_index,
                            output,
                            idx,
                        );
                    }
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
                        queue_address_credential(
                            &mut vkey_relation_map,
                            &mut queued_address_credential,
                            &mut queued_address,
                            cardano_transaction.database_index,
                            &reward_addr.to_address().to_bytes(),
                            &reward_addr.payment_cred(),
                            TxCredentialRelationValue::Withdrawal,
                            AddressCredentialRelationValue::PaymentKey,
                        );
                    }
                }
                TransactionBodyComponent::RequiredSigners(key_hashes) => {
                    for &signer in key_hashes.iter() {
                        let owner_credential =
                            pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(
                                signer,
                            )
                            .encode_fragment()
                            .unwrap();
                        vkey_relation_map.add_relation(
                            cardano_transaction.database_index,
                            &owner_credential.clone(),
                            TxCredentialRelationValue::RequiredSigner,
                        );
                    }
                }
                _ => (),
            }
        }

        for component in cardano_transaction.body.iter() {
            match component {
                TransactionBodyComponent::Inputs(inputs) => {
                    if cardano_transaction.is_valid {
                        queued_inputs.push((&inputs, cardano_transaction.database_index))
                    } else {
                        queued_unused_inputs.push((&inputs, cardano_transaction.database_index))
                    }
                }
                TransactionBodyComponent::Collateral(inputs) if !cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    if !cardano_transaction.is_valid {
                        queued_inputs.push((&inputs, cardano_transaction.database_index))
                    } else {
                        queued_unused_inputs.push((&inputs, cardano_transaction.database_index))
                    }
                }
                _ => (),
            };
        }
    }

    // 1) Insert addresses
    let (new_addresses, address_to_model_map) =
        crate::era_common::insert_addresses(&queued_address, txn).await?;
    perf_aggregator.addr_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 2) Insert outputs
    insert_outputs(&address_to_model_map, &queued_output, txn).await?;
    perf_aggregator.transaction_output_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 3) Insert inputs (note: inputs have to be added AFTER outputs added to DB)
    if !queued_inputs.is_empty() {
        let outputs_for_inputs =
            crate::era_common::get_outputs_for_inputs(&queued_inputs, txn).await?;
        let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

        add_input_relations(
            &mut vkey_relation_map,
            &queued_inputs,
            &outputs_for_inputs
                .iter()
                .map(|(output, _)| output)
                .collect(),
            &input_to_output_map,
        );
        crate::era_common::insert_inputs(&queued_inputs, &input_to_output_map, txn).await?;
    }

    // 4) Insert unused inputs
    if !queued_unused_inputs.is_empty() {
        let outputs_for_inputs =
            crate::era_common::get_outputs_for_inputs(&queued_unused_inputs, txn).await?;
        let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

        add_input_relations(
            &mut vkey_relation_map,
            &queued_unused_inputs,
            &outputs_for_inputs
                .iter()
                .map(|(output, _)| output)
                .collect(),
            &input_to_output_map,
        );
    }

    perf_aggregator.transaction_input_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 6) Insert stake credentials (note: has to be done after inputs as they may add new creds)
    let cred_to_model_map = insert_stake_credentials(
        &vkey_relation_map
            .0
            .values()
            .flat_map(|cred_to_model| cred_to_model.keys())
            .map(|pallas| pallas.to_vec())
            .collect(),
        txn,
    )
    .await?;
    perf_aggregator.stake_cred_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 6) Insert address credential relations
    insert_address_credential_relation(
        &cred_to_model_map,
        &new_addresses,
        &queued_address_credential,
        txn,
    )
    .await?;
    perf_aggregator.addr_cred_relation_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 7) Insert tx relations
    insert_tx_credentials(&vkey_relation_map, &cred_to_model_map, txn).await?;
    perf_aggregator.tx_credential_relation += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    Ok(())
}

async fn insert_tx_credentials(
    vkey_relation_map: &RelationMap,
    cred_to_model_map: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let mut models: Vec<TxCredentialActiveModel> = vec![];
    for (tx_id, mapping) in vkey_relation_map.0.iter() {
        models.extend(
            mapping
                .iter()
                .map(|(credential, relation)| TxCredentialActiveModel {
                    credential_id: Set(cred_to_model_map.get(&credential.to_vec()).unwrap().id),
                    tx_id: Set(*tx_id),
                    relation: Set(*relation),
                }),
        );
    }

    // no entries to add if tx only had Byron addresses
    if models.len() > 0 {
        TxCredential::insert_many(models).exec(txn).await?;
    }
    Ok(())
}

fn queue_witness(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    witness_set: &cardano_multiplatform_lib::TransactionWitnessSet,
) -> () {
    match witness_set.vkeys() {
        Some(vkeys) => {
            for i in 0..vkeys.len() {
                let vkey = vkeys.get(i);
                vkey_relation_map.add_relation(
                    tx_id,
                    &RelationMap::keyhash_to_pallas(&vkey.vkey().public_key().hash()).to_vec(),
                    TxCredentialRelationValue::Witness,
                );
            }
        }
        _ => (),
    };

    match witness_set.native_scripts() {
        Some(scripts) => {
            for i in 0..scripts.len() {
                let script = scripts.get(i);
                vkey_relation_map.add_relation(
                    tx_id,
                    &RelationMap::scripthash_to_pallas(
                        &script.hash(ScriptHashNamespace::NativeScript),
                    )
                    .to_vec(),
                    TxCredentialRelationValue::Witness,
                );

                let vkeys_in_script = RequiredSignersSet::from(&script);
                for vkey_in_script in vkeys_in_script {
                    vkey_relation_map.add_relation(
                        tx_id,
                        &RelationMap::keyhash_to_pallas(&vkey_in_script).to_vec(),
                        TxCredentialRelationValue::InNativeScript,
                    );
                }
            }
        }
        _ => (),
    };

    match witness_set.plutus_scripts() {
        Some(scripts) => {
            for i in 0..scripts.len() {
                let script = scripts.get(i);
                vkey_relation_map.add_relation(
                    tx_id,
                    &RelationMap::scripthash_to_pallas(&script.hash(ScriptHashNamespace::PlutusV1))
                        .to_vec(),
                    TxCredentialRelationValue::Witness,
                );
            }
        }
        _ => (),
    };
}

fn queue_certificate(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredential>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    tx_id: i64,
    cert: &Certificate,
) -> () {
    match cert {
        Certificate::StakeDelegation(credential, pool) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeDelegation,
            );

            vkey_relation_map.add_relation(
                tx_id,
                &RelationMap::keyhash_to_pallas(
                    &cardano_multiplatform_lib::crypto::Ed25519KeyHash::from_bytes(pool.to_vec())
                        .unwrap(),
                )
                .to_vec(),
                TxCredentialRelationValue::DelegationTarget,
            );
        }
        Certificate::StakeRegistration(credential) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeRegistration,
            );
        }
        Certificate::StakeDeregistration(credential) => {
            let credential = credential.encode_fragment().unwrap();

            vkey_relation_map.add_relation(
                tx_id,
                &credential,
                TxCredentialRelationValue::StakeDeregistration,
            );
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

            vkey_relation_map.add_relation(
                tx_id,
                &operator_credential,
                TxCredentialRelationValue::PoolOperator,
            );

            let reward_addr = RewardAddress::from_address(
                &cardano_multiplatform_lib::address::Address::from_bytes(reward_account.to_vec())
                    .unwrap(),
            )
            .unwrap();

            queue_address_credential(
                vkey_relation_map,
                queued_address_credential,
                queued_address,
                tx_id,
                &reward_addr.to_address().to_bytes(),
                &reward_addr.payment_cred(),
                TxCredentialRelationValue::PoolReward,
                AddressCredentialRelationValue::PaymentKey,
            );

            for &owner in pool_owners.iter() {
                let owner_credential =
                    pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(owner)
                        .encode_fragment()
                        .unwrap();

                vkey_relation_map.add_relation(
                    tx_id,
                    &owner_credential,
                    TxCredentialRelationValue::PoolOwner,
                );
            }
        }
        Certificate::PoolRetirement(key_hash, _) => {
            let operator_credential =
                pallas::ledger::primitives::alonzo::StakeCredential::AddrKeyhash(key_hash.clone())
                    .encode_fragment()
                    .unwrap();
            vkey_relation_map.add_relation(
                tx_id,
                &operator_credential,
                TxCredentialRelationValue::PoolOperator,
            );
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

                    vkey_relation_map.add_relation(
                        tx_id,
                        &credential,
                        TxCredentialRelationValue::MirRecipient,
                    );
                }
            }
            _ => {}
        },
    };
}

fn queue_address_credential(
    vkey_relation_map: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredential>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    tx_id: i64,
    address: &Vec<u8>,
    credential: &cardano_multiplatform_lib::address::StakeCredential,
    tx_relation: TxCredentialRelationValue,
    address_relation: AddressCredentialRelationValue,
) -> () {
    queued_address.insert(address.clone());
    vkey_relation_map.add_relation(tx_id, &credential.to_bytes(), tx_relation);
    queued_address_credential.insert(QueuedAddressCredential {
        address: address.clone(),
        stake_credential: credential.to_bytes(),
        address_relation: address_relation,
    });
}

fn queue_output(
    queued_credentials: &mut RelationMap,
    queued_address_credential: &mut BTreeSet<QueuedAddressCredential>,
    queued_address: &mut BTreeSet<Vec<u8>>,
    queued_output: &mut Vec<QueuedOutput>,
    tx_body: &TransactionBody,
    tx_id: i64,
    output: &alonzo::TransactionOutput,
    idx: usize,
) -> () {
    use cardano_multiplatform_lib::address::Address;
    let addr = Address::from_bytes(output.address.to_vec())
        .map_err(|e| panic!("{:?}{:?}", e, tx_body.to_hash().to_vec()))
        .unwrap();

    queued_output.push(QueuedOutput {
        payload: output.encode_fragment().unwrap(),
        address: addr.to_bytes(),
        tx_id,
        idx,
    });

    let tx_relation = TxCredentialRelationValue::Output;
    let address_relation = AddressCredentialRelationValue::PaymentKey;

    if let Some(base_addr) = BaseAddress::from_address(&addr) {
        // Payment Key
        {
            queue_address_credential(
                queued_credentials,
                queued_address_credential,
                queued_address,
                tx_id,
                &addr.to_bytes(),
                &base_addr.payment_cred(),
                tx_relation,
                address_relation,
            );
        }

        // Stake Key
        {
            queue_address_credential(
                queued_credentials,
                queued_address_credential,
                queued_address,
                tx_id,
                &addr.to_bytes(),
                &base_addr.stake_cred(),
                TxCredentialRelationValue::OutputStake,
                AddressCredentialRelationValue::StakeKey,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &reward_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else if let Some(_) = ByronAddress::from_address(&addr) {
        queued_address.insert(addr.to_bytes());
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &enterprise_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
        queue_address_credential(
            queued_credentials,
            queued_address_credential,
            queued_address,
            tx_id,
            &addr.to_bytes(),
            &ptr_addr.payment_cred(),
            tx_relation,
            address_relation,
        );
    } else {
        panic!("Unexpected address type {}", hex::encode(addr.to_bytes()));
    }
}

async fn insert_stake_credentials(
    deduplicated_creds: &BTreeSet<Vec<u8>>,
    txn: &DatabaseTransaction,
) -> Result<BTreeMap<Vec<u8>, StakeCredentialModel>, DbErr> {
    let mut result_map = BTreeMap::<Vec<u8>, StakeCredentialModel>::default();

    if deduplicated_creds.len() == 0 {
        return Ok(result_map);
    }

    // 1) Add credentials that were already in the DB
    {
        let mut found_credentials = StakeCredential::find()
            .filter(
                Condition::any()
                    .add(StakeCredentialColumn::Credential.is_in(deduplicated_creds.clone())),
            )
            .all(txn)
            .await?;

        result_map.extend(
            found_credentials
                .drain(..)
                .map(|model| (model.credential.clone(), model)),
        );
    }

    // 2) Add credentials that weren't in the DB
    {
        // check which credentials weren't found in the DB and prepare to add them
        let credentials_to_add: Vec<StakeCredentialActiveModel> = deduplicated_creds
            .iter()
            .filter(|&credential| !result_map.contains_key(credential))
            .map(|credential| StakeCredentialActiveModel {
                credential: Set(credential.to_vec()),
                ..Default::default()
            })
            .collect();

        // add the new entires into the DB, then add them to our result mapping
        if !credentials_to_add.is_empty() {
            let mut additions = StakeCredential::insert_many(credentials_to_add)
                .exec_many_with_returning(txn)
                .await?;
            additions.drain(..).for_each(|model| {
                result_map.insert(model.credential.clone(), model);
            });
        }
    }

    Ok(result_map)
}

async fn insert_address_credential_relation(
    cred_to_model_map: &BTreeMap<Vec<u8>, StakeCredentialModel>,
    new_addresses: &Vec<AddressModel>,
    queued_address_credential: &BTreeSet<QueuedAddressCredential>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if queued_address_credential.is_empty() {
        return Ok(());
    }

    let address_map: BTreeMap<&Vec<u8>, &AddressModel> = new_addresses
        .iter()
        .map(|addr| (&addr.payload, addr))
        .collect();

    let mut to_add: Vec<AddressCredentialActiveModel> = vec![];
    for entry in queued_address_credential {
        match address_map.get(&entry.address) {
            Some(&address_model) => {
                to_add.push(AddressCredentialActiveModel {
                    credential_id: Set(cred_to_model_map.get(&entry.stake_credential).unwrap().id),
                    address_id: Set(address_model.id),
                    relation: Set(entry.address_relation as i32),
                });
            }
            // we can ignore addresses we've already seen before
            None => {}
        }
    }

    if to_add.len() > 0 {
        AddressCredential::insert_many(to_add).exec(txn).await?;
    }

    Ok(())
}

async fn insert_outputs(
    address_to_model_map: &BTreeMap<Vec<u8>, AddressModel>,
    queued_output: &Vec<QueuedOutput>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if queued_output.is_empty() {
        return Ok(());
    };

    TransactionOutput::insert_many(queued_output.iter().map(|entry| {
        TransactionOutputActiveModel {
            address_id: Set(address_to_model_map
                .get(get_truncated_address(&entry.address))
                .unwrap()
                .id),
            tx_id: Set(entry.tx_id),
            payload: Set(entry.payload.clone()),
            output_index: Set(entry.idx as i32),
            ..Default::default()
        }
    }))
    .exec(txn)
    .await?;

    Ok(())
}

fn add_input_relations(
    vkey_relation_map: &mut RelationMap,
    inputs: &Vec<(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )>,
    outputs: &Vec<&TransactionOutputModel>,
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, i64>>,
) -> () {
    let mut output_to_input_tx = BTreeMap::<i64, i64>::default();
    for input_tx_pair in inputs.iter() {
        for input in input_tx_pair.0.iter() {
            match input_to_output_map.get(&input.transaction_id.to_vec()) {
                Some(entry_for_tx) => {
                    let output_id = entry_for_tx[&(input.index as i64)];
                    output_to_input_tx.insert(output_id, input_tx_pair.1);
                }
                None => {
                    println!("tx: {}", hex::encode(input.transaction_id.to_vec()));
                    panic!();
                }
            }
        }
    }

    outputs.iter().for_each(|&output| {
        match &cardano_multiplatform_lib::TransactionOutput::from_bytes(output.payload.clone()) {
            Ok(payload) => {
                add_input_cred_relation(
                    vkey_relation_map,
                    output_to_input_tx[&output.id],
                    &payload.address(),
                    TxCredentialRelationValue::Input,
                    TxCredentialRelationValue::InputStake,
                );
            }
            Err(_e) => {
                // https://github.com/dcSpark/cardano-multiplatform-lib/issues/61
            }
        };
    });
}

fn add_input_cred_relation(
    vkey_relation_map: &mut RelationMap,
    tx_id: i64,
    addr: &cardano_multiplatform_lib::address::Address,
    input_relation: TxCredentialRelationValue,
    input_stake_relation: TxCredentialRelationValue,
) -> () {
    if let Some(base_addr) = BaseAddress::from_address(&addr) {
        // Payment Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.payment_cred().to_bytes(),
                input_relation,
            );
        }

        // Stake Key
        {
            vkey_relation_map.add_relation(
                tx_id,
                &base_addr.stake_cred().to_bytes(),
                input_stake_relation,
            );
        }
    } else if let Some(reward_addr) = RewardAddress::from_address(&addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &reward_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if let Some(_) = ByronAddress::from_address(&addr) {
        // Byron address has no credentials
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&addr) {
        vkey_relation_map.add_relation(
            tx_id,
            &enterprise_addr.payment_cred().to_bytes(),
            input_relation,
        );
    } else if let Some(ptr_addr) = PointerAddress::from_address(&addr) {
        vkey_relation_map.add_relation(tx_id, &ptr_addr.payment_cred().to_bytes(), input_relation);
    } else {
        panic!("Unexpected address type {}", hex::encode(addr.to_bytes()));
    }
}
