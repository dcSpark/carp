/* 
@name AssetUtxos 
@param fingerprints -> (...)
@param policyIds -> (...)
*/
SELECT
	ENCODE("Transaction".HASH, 'hex') TX,
	ENCODE("Block".HASH, 'hex') AS "block!",
	json_agg(json_build_object(
		'outputIndex', "TransactionOutput".OUTPUT_INDEX,
		'outputTxHash', ENCODE(TXO.HASH, 'hex'),
		'cip14Fingerprint', ENCODE("NativeAsset".CIP14_FINGERPRINT, 'hex'),
		'policyId', ENCODE("NativeAsset".POLICY_ID, 'hex'),
		'assetName', ENCODE("NativeAsset".ASSET_NAME, 'hex'),
		'amount', "AssetUtxo".AMOUNT,
		'slot', "Block".SLOT,
		'addressRaw', ENCODE("Address".PAYLOAD, 'hex')
	)) as "payload!"
FROM "AssetUtxo"
JOIN "Transaction" ON "AssetUtxo".TX_ID = "Transaction".ID
JOIN "TransactionOutput" ON "AssetUtxo".UTXO_ID = "TransactionOutput".ID
JOIN "Transaction" TXO ON "TransactionOutput".TX_ID = TXO.ID
JOIN "Address" ON "Address".id = "TransactionOutput".address_id
JOIN "NativeAsset" ON "AssetUtxo".ASSET_ID = "NativeAsset".ID
JOIN "Block" ON "Transaction".BLOCK_ID = "Block".ID
WHERE 
	("NativeAsset".CIP14_FINGERPRINT IN :fingerprints
		OR "NativeAsset".POLICY_ID IN :policyIds
	) AND
	"Transaction".id > :after_tx_id! AND
	"Transaction".id <= :until_tx_id!
GROUP BY ("Block".ID, "Transaction".ID)
ORDER BY "Transaction".ID ASC
LIMIT :limit!;
