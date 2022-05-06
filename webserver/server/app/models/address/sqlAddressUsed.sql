/* @name sqlAddressUsed */
SELECT DISTINCT "Address".payload
FROM "Address"
INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
WHERE
  "Address".payload = ANY (:addresses)
  AND
  /* is within untilBlock (inclusive) */
  "Transaction".block_id <= (:until_block_id)
  AND (
    /* 
      * Either:
      * 1: comes in block strict after the "after" field
    */
    "Transaction".block_id > (:after_block_id)
      OR
    /* 2) Is in the same block as the "after" field, but is tx that appears afterwards */
    ("Transaction".block_id = (:after_block_id) AND "Transaction".id > (:after_tx_id))
  );