/* @name sqlAddressUsed */
WITH
  address_row AS (
    SELECT *
    FROM "Address"
    WHERE "Address".payload = ANY (:addresses)
  ),
  outputs AS (
    SELECT DISTINCT address_row.payload
    FROM "TransactionOutput"
    INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
    WHERE
      "TransactionOutput".tx_id <= (:until_tx_id)
      AND
      "TransactionOutput".tx_id > (:after_tx_id)
  ),
  inputs AS (
    SELECT DISTINCT address_row.payload
    FROM "TransactionInput"
    INNER JOIN (
      SELECT "TransactionOutput".id, "TransactionOutput".address_id
      FROM "TransactionOutput"
      INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
      WHERE
        "TransactionOutput".tx_id <= (:until_tx_id)
    ) spent_utxos ON "TransactionInput".utxo_id = spent_utxos.id
    INNER JOIN address_row ON spent_utxos.address_id = address_row.id
    WHERE
      "TransactionInput".tx_id <= (:until_tx_id)
      AND
      "TransactionInput".tx_id > (:after_tx_id)
  )
SELECT DISTINCT all_address.payload
FROM (SELECT * FROM inputs UNION ALL SELECT * from outputs) all_address;