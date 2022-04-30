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
      WHERE
        "Address".payload = ANY (:addresses)
        AND
        /* is within untilBlock (inclusive) */
        "Block".id <= (:until_block_id)
        and (
          /* 
           * Either:
           * 1: comes in block strict after the "after" field
          */
          "Block".id > (:after_block_id)
            or
          /* 2) Is in the same block as the "after" field, but is tx that appears afterwards */
          ("Block".id = (:after_block_id) and "Transaction".id > (:after_tx_id))
        ) 
      ORDER BY
        "Block".height ASC,
        "Transaction".tx_index ASC
      LIMIT (:limit);