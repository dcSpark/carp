/* @name sqlHistoryForCredentials */
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
      FROM "StakeCredential"
      INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
      INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
      INNER JOIN "Block" ON "Transaction".block_id = "Block".id
      WHERE "StakeCredential".credential = ANY (:credentials)
      ORDER BY
        "Block".height ASC,
        "Transaction".tx_index ASC
      LIMIT (:limit);
