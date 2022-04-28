/* @name sqlHistoryForAddresses */
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
      FROM "Address"
      INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
      INNER JOIN "Transaction" ON "TransactionOutput".tx_id = "Transaction".id
      INNER JOIN "Block" ON "Transaction".block_id = "Block".id
      WHERE "Address".payload = ANY (:addresses)
      ORDER BY
        "Block".height ASC,
        "Transaction".tx_index ASC
      LIMIT (:limit);