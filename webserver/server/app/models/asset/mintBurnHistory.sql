/*
@name sqlMintBurnRange
*/
SELECT
	ENCODE("Transaction".HASH, 'hex') "tx!",
	ENCODE("Block".HASH, 'hex') AS "block!",
	"Block".slot AS action_slot,
	ENCODE("TransactionMetadata".payload, 'hex') as action_tx_metadata,
	json_agg(json_build_object(
        'amount', "AssetMint".amount::text,
        'policyId', encode("NativeAsset".policy_id, 'hex'),
        'assetName', encode("NativeAsset".asset_name, 'hex')
	)) as "payload!",
	array_agg(DISTINCT "TransactionOutput".payload) as output_payloads,
	array_agg(DISTINCT input_outputs.payload) input_payloads
FROM "AssetMint"
         LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
         JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
         JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
		 LEFT JOIN "TransactionOutput" ON "TransactionOutput".tx_id = "Transaction".id
		 LEFT JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
		 LEFT JOIN "TransactionOutput" input_outputs ON "TransactionInput".utxo_id = input_outputs.id
WHERE
	"Transaction".id > :after_tx_id! AND
	"Transaction".id <= :until_tx_id!
GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
ORDER BY "Transaction".id ASC
LIMIT :limit!;

/*
@name sqlMintBurnRangeByPolicyIds
@param policy_ids -> (...)
*/
SELECT
	ENCODE("Transaction".HASH, 'hex') "tx!",
	ENCODE("Block".HASH, 'hex') AS "block!",
	"Block".slot AS action_slot,
	ENCODE("TransactionMetadata".payload, 'hex') as action_tx_metadata,
	json_agg(json_build_object(
        'amount', "AssetMint".amount::text,
        'policyId', encode("NativeAsset".policy_id, 'hex'),
        'assetName', encode("NativeAsset".asset_name, 'hex')
	)) as "payload!",
	array_agg(DISTINCT "TransactionOutput".payload) as output_payloads,
	array_agg(DISTINCT input_outputs.payload) input_payloads
FROM "AssetMint"
         LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
         JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
         JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
		 LEFT JOIN "TransactionOutput" ON "TransactionOutput".tx_id = "Transaction".id
		 LEFT JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
		 LEFT JOIN "TransactionOutput" input_outputs ON "TransactionInput".utxo_id = input_outputs.id
WHERE
	"Transaction".id > :after_tx_id! AND
	"Transaction".id <= :until_tx_id!
    AND "NativeAsset".policy_id IN :policy_ids!
GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
ORDER BY "Transaction".id ASC
LIMIT :limit!;