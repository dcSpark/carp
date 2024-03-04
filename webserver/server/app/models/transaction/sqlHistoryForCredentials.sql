/* @name sqlHistoryForCredentials */
WITH
  tx_relations AS (
    SELECT DISTINCT ON ("TxCredentialRelation".tx_id) "TxCredentialRelation".tx_id
    FROM "StakeCredential"
    INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
    WHERE
      "StakeCredential".credential = ANY (:credentials)
      AND
      ("TxCredentialRelation".relation & (:relation)) > 0
      AND
      /* is within untilBlock (inclusive) */
      "TxCredentialRelation".tx_id <= (:until_tx_id)
      AND 
      "TxCredentialRelation".tx_id > (:after_tx_id)
    ORDER BY "TxCredentialRelation".tx_id ASC
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
        "Block".height,
        "TransactionMetadata".payload AS metadata,
        json_agg(DISTINCT "Address".PAYLOAD) input_addresses
FROM tx_relations
INNER JOIN "Transaction" ON tx_relations.tx_id = "Transaction".id
INNER JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
INNER JOIN "Address" ON "Address".id = "TransactionInput".address_id
LEFT JOIN "TransactionMetadata" ON "Transaction".id = "TransactionMetadata".tx_id
INNER JOIN "Block" ON "Transaction".block_id = "Block".id
GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id;

