use crate::perf_aggregator::PerfAggregator;
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

/// Oura & CML have a bug where the tx hash is wrong
/// if the cbor include a larger-than-necessary int representation
/// Until we get a proper fix, we create a mapping of broken hash to real hash
// static broken_hash_mapping: Lazy<BTreeMap<Vec<u8>, Vec<u8>>> = Lazy::new(|| {
//     let mut mapping = BTreeMap::<Vec<u8>, Vec<u8>>::default();
//     mapping.insert(
//         hex::decode("80aa254b951c57b9cc3cac7fd87da8021839ee47d120c0e7ae228cebe81c6754").unwrap(),
//         hex::decode("85d476f64056d8cb776239147ad4430711cc69286823ec888c59776bf8ffd794").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("8f8a36bef60a4e92f5f345c4544dd454bbf1824361ae9bf6c1e9221bd24c4f39").unwrap(),
//         hex::decode("b6d54d7b7ea398a923789bbec8838ad9060fa09fcfd2085aad241963e2b1ab27").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("d10ec76956c3f58e18c9a8bc680cf7311a6e132cf73e638060c70499aa3bd1eb").unwrap(),
//         hex::decode("68ec93da477aec3cf91e2de5ed55113a93a3b523a016a9c9a03bde32533edcd3").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("bbf040785dd06fb3f595af0a23d9f17f9ca0393d39012c8bdef42a286cd047a5").unwrap(),
//         hex::decode("b85ca66fdfebd3bc1cc14fa13bb03ffb25be7e75721888d5fcf82175efa83ddf").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("19d5925fe8021913db3a5b7d91c323fb02ba5db666907d7461c3f8889e431aa0").unwrap(),
//         hex::decode("c313dd7f7f084d3a70c692fb08db109d4637261ccb6d8b38edbeacdb42f626d7").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("85ca0ef7d48d0a9d77e4db9581ef8da339bad2b7d730ec07f4cb3bf05df8ffc2").unwrap(),
//         hex::decode("716a5e92c50bf7d896cb79b338a0aedd4b7b4dd0bc3ff29212cf906a51715a71").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("6744f9f865db1dbf42bbf7be47902149b49d678d799a6ac0cb4ce7ace311eefd").unwrap(),
//         hex::decode("5efde14726458207dbd96fb6a1ffd0528d47f51f55f4e0c34d52a8e004a6b25a").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("7212feef21477ee683605656c2f7a161e45a3388e1e2339c931e70106e1367c0").unwrap(),
//         hex::decode("56c4c6e56790f06a3cc54baa7becb093ea40608a3e870ee749411438d761689e").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("6fdc2a2c06a7487992cf505713076ec64d517ff23a63c468fb8ab91f1603a645").unwrap(),
//         hex::decode("1d31bfa575281e594440c7adfec0e0c676f3a7a4112b07092ce28b05ab0a39fd").unwrap(),
//     );
//     mapping.insert(
//         hex::decode("16a65f7f4eff43dcb7abe4daa8a74d8d2d038548659981f61526cd61c7763082").unwrap(),
//         hex::decode("91a99d669384cd60f1a23cbec7e8c956ead8b812f57eaa59433e5175c4eed7e7").unwrap(),
//     );

//     mapping
// });

// fn replace_inputs(
//     inputs: &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
// ) -> Vec<pallas::ledger::primitives::alonzo::TransactionInput> {
//     inputs
//         .iter()
//         .map(
//             |input| match broken_hash_mapping.get(&input.transaction_id.to_vec()) {
//                 Some(new_hash) => pallas::ledger::primitives::alonzo::TransactionInput {
//                     transaction_id: RelationMap::bytes_to_pallas(new_hash),
//                     index: input.index,
//                 },
//                 None => pallas::ledger::primitives::alonzo::TransactionInput::decode_fragment(
//                     &input.encode_fragment().unwrap(),
//                 )
//                 .unwrap(),
//             },
//         )
//         .collect()
// }

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
                TransactionBodyComponent::Inputs(inputs) if cardano_transaction.is_valid => {
                    queued_inputs.push((&inputs, cardano_transaction.database_index))
                }
                TransactionBodyComponent::Collateral(inputs) if !cardano_transaction.is_valid => {
                    // note: we consider collateral as just another kind of input instead of a separate table
                    // you can use the is_valid field to know what kind of input it actually is
                    queued_inputs.push((&inputs, cardano_transaction.database_index))
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
    {
        // let mapped_inputs: Vec<_> = queued_inputs
        //     .iter()
        //     .map(|&(inputs, _)| replace_inputs(inputs))
        //     .collect();
        // let fixed_queued_inputs: Vec<_> = queued_inputs
        //     .iter()
        //     .enumerate()
        //     .map(|(i, &(_, output_idx))| (&mapped_inputs[i], output_idx))
        //     .collect();
        let outputs_for_inputs =
            crate::era_common::get_outputs_for_inputs(&queued_inputs, txn).await?;
        let input_to_output_map = crate::era_common::gen_input_to_output_map(&outputs_for_inputs);

        let (relation_result, input_result) = futures::future::join(
            add_input_relations(
                &mut vkey_relation_map,
                &queued_inputs,
                &outputs_for_inputs,
                &input_to_output_map,
                txn,
            ),
            crate::era_common::insert_inputs(&queued_inputs, &input_to_output_map, txn),
        )
        .await;
        relation_result?;
        input_result?;
    }
    perf_aggregator.transaction_input_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 4) Insert stake credentials (note: has to be done after inputs as they may add new creds)
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

    // 5) Insert address credential relations
    insert_address_credential_relation(
        &cred_to_model_map,
        &new_addresses,
        &queued_address_credential,
        txn,
    )
    .await?;
    perf_aggregator.addr_cred_relation_insert += time_counter.elapsed();
    *time_counter = std::time::Instant::now();

    // 6) Insert tx relations
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
                tx_relation,
                AddressCredentialRelationValue::StakeKey,
            );
        }
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
            address_id: Set(address_to_model_map.get(&entry.address).unwrap().id),
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

async fn add_input_relations(
    vkey_relation_map: &mut RelationMap,
    inputs: &Vec<(
        &Vec<pallas::ledger::primitives::alonzo::TransactionInput>,
        i64,
    )>,
    outputs_for_inputs: &Vec<(TransactionOutputModel, Vec<TransactionModel>)>,
    input_to_output_map: &BTreeMap<&Vec<u8>, BTreeMap<i64, i64>>,
    txn: &DatabaseTransaction,
) -> Result<(), DbErr> {
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

    let shelley_output_ids: Vec<i64> = outputs_for_inputs
        .iter()
        // Byron addresses don't contain stake credentials, so we can skip them
        .filter(|&tx_output| {
            let is_byron = match cardano_multiplatform_lib::TransactionOutput::from_bytes(
                tx_output.0.payload.clone(),
            ) {
                Ok(parsed_output) => parsed_output.address().as_byron().is_some(),
                // TODO: remove this once we've parsed the genesis block correctly instead of inserting dummy data
                Err(_) => true,
            };
            !is_byron
        })
        .map(|output| output.0.id)
        .collect();

    if shelley_output_ids.len() > 0 {
        // get stake credentials for the outputs that were consumed
        // note: this may return duplicates if the same credential is used
        // as both the payment key and the staking key of a base address
        let related_credentials = StakeCredential::find()
            .inner_join(AddressCredential)
            .join(
                JoinType::InnerJoin,
                AddressCredentialRelation::Address.def(),
            )
            .join(
                JoinType::InnerJoin,
                AddressRelation::TransactionOutput.def(),
            )
            .filter(
                Condition::any().add(TransactionOutputColumn::Id.is_in(shelley_output_ids.clone())),
            )
            // we need to know which OutputId every credential is for so we can know which tx these creds are related to
            .select_with(TransactionOutput)
            // TODO: we only actually need these columns, but sea-orm returns the full join
            // .column(StakeCredentialColumn::Id)
            // .column(StakeCredentialColumn::Credential)
            // .column(TransactionOutputColumn::Id)
            .all(txn)
            .await?;

        // 4) Associate the stake credentials to this transaction
        if related_credentials.len() > 0 {
            for stake_credentials in &related_credentials {
                // recall: the same stake credential could have shown up in multiple outputs
                for output in stake_credentials.1.iter() {
                    vkey_relation_map.add_relation(
                        output_to_input_tx[&output.id],
                        &stake_credentials.0.credential,
                        TxCredentialRelationValue::Input,
                    );
                }
            }
        }
    }

    Ok(())
}
