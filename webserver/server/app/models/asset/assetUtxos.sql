/* 
@name AssetUtxos 
@param fingerprints -> (...)
*/
SELECT ENCODE(TXO.HASH,
        'hex') OUTPUT_TX_HASH,
    "TransactionOutput".OUTPUT_INDEX,
	"NativeAsset".CIP14_FINGERPRINT,
	"AssetUtxo".AMOUNT,
	"Block".SLOT,
	ENCODE("Transaction".HASH,
        'hex') TX_HASH,
	"Address".PAYLOAD ADDRESS_RAW
FROM "AssetUtxo"
JOIN "Transaction" ON "AssetUtxo".TX_ID = "Transaction".ID
JOIN "TransactionOutput" ON "AssetUtxo".UTXO_ID = "TransactionOutput".ID
JOIN "Transaction" TXO ON "TransactionOutput".TX_ID = TXO.ID
JOIN "Address" ON "Address".id = "TransactionOutput".address_id
JOIN "NativeAsset" ON "AssetUtxo".ASSET_ID = "NativeAsset".ID
JOIN "Block" ON "Transaction".BLOCK_ID = "Block".ID
WHERE 
	"NativeAsset".CIP14_FINGERPRINT IN :fingerprints! AND
	"Block".SLOT > :min_slot! AND
	"Block".SLOT <= :max_slot!
ORDER BY "Transaction".ID, "AssetUtxo".ID ASC;
