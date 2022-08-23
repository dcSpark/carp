/* @name sqlTransactionOutput */
WITH pointers AS (
  SELECT tx_hash, output_index
  FROM
    unnest(
      (:tx_hash)::bytea[],
      (:output_index)::int[]
    ) x(tx_hash,output_index)
)
SELECT
  "TransactionOutput".payload as utxo_payload,
  "Transaction".is_valid,
  "Transaction".tx_index,
  "Transaction".hash,
  "Block".hash AS block_hash,
  "Block".epoch,
  "Block".slot,
  "Block".era,
  "Block".height,
  "TransactionOutput".output_index
FROM
  "Transaction"
  INNER JOIN "TransactionOutput" ON "Transaction".id = "TransactionOutput".tx_id
  INNER JOIN "Block" on "Block".id = "Transaction".block_id
WHERE ("Transaction".hash, "TransactionOutput".output_index) in (SELECT tx_hash, output_index FROM pointers);
