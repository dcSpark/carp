/* @name sqlHistoryForAddresses */
WITH
  address_row AS (
    SELECT *
    FROM "Address"
    WHERE "Address".payload = ANY (:addresses)
  ),
  outputs AS (
        SELECT DISTINCT ON ("TransactionOutput".tx_id) "TransactionOutput".tx_id
        FROM "TransactionOutput"
        INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
        WHERE
          "TransactionOutput".tx_id <= (:until_tx_id)
          AND
          "TransactionOutput".tx_id > (:after_tx_id)
        ORDER BY "TransactionOutput".tx_id ASC
        LIMIT (:limit)
  ),
  inputs AS (
        SELECT DISTINCT ON ("TransactionInput".tx_id) "TransactionInput".tx_id
        FROM "TransactionInput"
        INNER JOIN address_row ON "TransactionInput".address_id = address_row.id
        WHERE
          "TransactionInput".tx_id <= (:until_tx_id)
          AND
          "TransactionInput".tx_id > (:after_tx_id)
        ORDER BY "TransactionInput".tx_id ASC
        LIMIT (:limit)
  ),
  ref_inputs AS (
        SELECT DISTINCT ON ("TransactionReferenceInput".tx_id) "TransactionReferenceInput".tx_id
        FROM "TransactionReferenceInput"
        INNER JOIN address_row ON "TransactionReferenceInput".address_id = address_row.id
        WHERE
          "TransactionReferenceInput".tx_id <= (:until_tx_id)
          AND
          "TransactionReferenceInput".tx_id > (:after_tx_id)
        ORDER BY "TransactionReferenceInput".tx_id ASC
        LIMIT (:limit)
  )
SELECT "Transaction".id,
        "Transaction".payload,
        "Transaction".hash,
        "Transaction".tx_index,
        "Transaction".is_valid,
        "Block".hash AS block_hash,
        "Block".epoch,
        "Block".slot,
        "Block".era,
        "Block".height
FROM "Transaction"
INNER JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE "Transaction".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)
ORDER BY "Transaction".id ASC
LIMIT (:limit);