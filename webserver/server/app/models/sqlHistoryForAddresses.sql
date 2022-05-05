/* @name sqlHistoryForAddresses */
WITH related_txs AS (
  SELECT DISTINCT ON ("Transaction".id) "Transaction".*
  FROM "Address"
  INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
  LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
  INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
  WHERE "Address".payload = ANY (:addresses)
)
SELECT related_txs.id,
        related_txs.payload,
        related_txs.hash,
        related_txs.tx_index,
        related_txs.is_valid,
        "Block".hash AS block_hash,
        "Block".epoch,
        "Block".slot,
        "Block".era,
        "Block".height
      FROM related_txs
      INNER JOIN "Block" ON related_txs.block_id = "Block".id
      WHERE
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
          ("Block".id = (:after_block_id) and related_txs.id > (:after_tx_id))
        ) 
      ORDER BY
        "Block".height ASC,
        related_txs.tx_index ASC
      LIMIT (:limit);