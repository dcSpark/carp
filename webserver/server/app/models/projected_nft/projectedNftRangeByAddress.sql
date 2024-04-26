/*
@name sqlProjectedNftRangeByAddress
*/
SELECT
    json_agg(json_build_object(
        'ownerAddress', encode("ProjectedNFT".owner_address, 'hex'),
        'previousUtxoTxHash', encode("ProjectedNFT".previous_utxo_tx_hash, 'hex'),
        'previousTxOutputIndex', "ProjectedNFT".previous_utxo_tx_output_index,
        'actionOutputIndex', CASE
            WHEN "TransactionOutput".output_index = NULL THEN NULL
            ELSE "TransactionOutput".output_index
            END,
        'policyId', "ProjectedNFT".policy_id,
        'assetName', "ProjectedNFT".asset_name,
        'amount', "ProjectedNFT".amount,
        'status', CASE
            WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
            WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
            WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
            ELSE 'Invalid'
            END,
        'plutusDatum', encode("ProjectedNFT".plutus_datum, 'hex'),
        'forHowLong', "ProjectedNFT".for_how_long,
        'actionSlot', "Block".slot
    )) as "payload!",
    encode("Block".hash, 'hex') as "block!",
    encode("Transaction".hash, 'hex') as "tx_id!"
FROM "ProjectedNFT"
         LEFT JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".hololocker_utxo_id
         JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
    encode("ProjectedNFT".owner_address, 'hex') = :owner_address! AND
	"Transaction".id > :after_tx_id! AND
	"Transaction".id <= :until_tx_id!
GROUP BY ("Block".id, "Transaction".id)
ORDER BY ("Block".height, "Transaction".tx_index) ASC
LIMIT :limit!;
