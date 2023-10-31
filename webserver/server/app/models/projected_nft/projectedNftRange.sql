/*
@name sqlProjectedNftRange
*/
SELECT
    CASE
        WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
        WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
        WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
        ELSE 'Invalid'
        END AS status,
    "ProjectedNFT".asset as asset,
    "ProjectedNFT".amount as amount,
    encode("Transaction".hash, 'hex') as tx_id,
    "TransactionOutput".output_index as output_index,
    encode("ProjectedNFT".plutus_datum, 'hex') as plutus_datum,
    "Block".slot
FROM "ProjectedNFT"
         JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".utxo_id
         JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
        "Block".slot > :min_slot!
    AND "Block".slot <= :max_slot!
ORDER BY ("Block".height, "Transaction".tx_index, "TransactionOutput".output_index) ASC;