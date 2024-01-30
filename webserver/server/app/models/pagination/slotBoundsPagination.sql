 /* @name slotBoundsPagination */
WITH
min_hash AS
(
         SELECT   COALESCE("Transaction".id, -1) AS min_tx_id,
                  slot                           AS min_slot
         FROM     "Transaction"
         JOIN     "Block"
         ON       "Block".id = "Transaction".block_id
         WHERE    slot <= :low!
         ORDER BY "Block".id DESC,
                  "Transaction".id DESC
         LIMIT 1
),
max_hash AS
(
         SELECT   slot                                AS max_slot,
                  COALESCE(Max("Transaction".id), -2) AS max_tx_id
         FROM     "Transaction"
         JOIN     "Block"
         ON       "Transaction".block_id = "Block".id
         WHERE    slot <= :high!
         GROUP BY "Block".id
         ORDER BY "Block".id DESC
         LIMIT 1
)
SELECT    *
FROM      min_hash
LEFT JOIN max_hash
ON        1 = 1; 