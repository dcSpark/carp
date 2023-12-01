/*
@name sqlProjectedNftRange
*/
SELECT
    encode("ProjectedNFT".owner_address, 'hex') as owner_address,

    encode("ProjectedNFT".previous_utxo_tx_hash, 'hex') as previous_tx_hash,
    "ProjectedNFT".previous_utxo_tx_output_index as previous_tx_output_index,

    CASE
        WHEN "TransactionOutput".output_index = NULL THEN NULL
        ELSE "TransactionOutput".output_index
        END AS action_output_index,

    encode("Transaction".hash, 'hex') as action_tx_id,

    "ProjectedNFT".asset as asset,
    "ProjectedNFT".amount as amount,

    CASE
        WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
        WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
        WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
        ELSE 'Invalid'
        END AS status,

    encode("ProjectedNFT".plutus_datum, 'hex') as plutus_datum,
    "ProjectedNFT".for_how_long as for_how_long,

    "Block".slot as action_slot
FROM "ProjectedNFT"
         LEFT JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".hololocker_utxo_id
         JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
        "Block".slot > :min_slot!
    AND "Block".slot <= :max_slot!
ORDER BY ("Block".height, "Transaction".tx_index) ASC;
