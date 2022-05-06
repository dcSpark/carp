/* @name sqlTransactionOutput */
WITH pointers AS (
  SELECT tx_hash, output_index
  FROM
    unnest(
      (:tx_hash)::bytea[],
      (:output_index)::int[]
    ) x(tx_hash,output_index)
)
SELECT "TransactionOutput".payload
FROM
  "Transaction"
  INNER JOIN "TransactionOutput" ON "Transaction".id = "TransactionOutput".tx_id
WHERE ("Transaction".hash, "TransactionOutput".output_index) in (SELECT tx_hash, output_index FROM pointers);