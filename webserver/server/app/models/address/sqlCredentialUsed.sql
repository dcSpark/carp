/* @name sqlCredentialUsed */
SELECT DISTINCT "StakeCredential".credential
FROM "StakeCredential"
INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
INNER JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
  "StakeCredential".credential = ANY (:credentials)
  AND
  ("TxCredentialRelation".relation & (:relation)) > 0
  AND
  /* is within untilBlock (inclusive) */
  "Transaction".block_id <= (:until_block_id)
  AND (
    /* 
      * Either:
      * 1: comes in block strict after the "after" field
    */
    "Transaction".block_id > (:after_block_id)
      OR
    /* 2) Is in the same block as the "after" field, but is tx that appears afterwards */
    ("Transaction".block_id = (:after_block_id) AND "Transaction".id > (:after_tx_id))
  );