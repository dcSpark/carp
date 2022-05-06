/* @name pageStartByHash */
SELECT
  "Block".id as after_block_id,
  "Transaction".id as after_tx_id
FROM "Transaction" INNER JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
  "Block".hash = (:after_block)
  AND 
  "Transaction".hash = (:after_tx);