/* @name sqlTransactionBeforeBlock */
WITH block_info AS (
  SELECT "Block".id as until_block_id
  FROM "Block"
  WHERE "Block".hash = (:until_block)
)
SELECT "Transaction".id
FROM "Transaction"
WHERE "Transaction".block_id <= (SELECT until_block_id FROM block_info)
/* stackoverflow.com/questions/21385555/postgresql-query-very-slow-with-limit-1 */
/* dev.to/cassidycodes/one-weird-trick-for-speeding-up-order-by-that-you-probably-shouldn-t-use-4pk5 */
ORDER BY "Transaction".block_id DESC, "Transaction".id DESC
LIMIT 1;
