/* @name sqlAddressUsed */
WITH address_tx_relations AS (
  SELECT "Address".*, "Transaction".id as tx_id, "Transaction".block_id
  FROM "Address"
  INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
  LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
  INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
  WHERE "Address".payload = ANY (:addresses)
)
SELECT DISTINCT address_tx_relations.payload
FROM address_tx_relations
INNER JOIN "Block" ON address_tx_relations.block_id = "Block".id
WHERE
  /* is within untilBlock (inclusive) */
  address_tx_relations.block_id <= (:until_block_id)
  and (
    /* 
      * Either:
      * 1: comes in block strict after the "after" field
    */
    address_tx_relations.block_id > (:after_block_id)
      or
    /* 2) Is in the same block as the "after" field, but is tx that appears afterwards */
    (address_tx_relations.block_id = (:after_block_id) and address_tx_relations.tx_id > (:after_tx_id))
  );