/* @name cursorFromTxId */
SELECT "Transaction".hash as tx_hash, "Block".hash as block_hash
FROM "Transaction"
INNER JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE "Transaction".id = (:tx_id);
