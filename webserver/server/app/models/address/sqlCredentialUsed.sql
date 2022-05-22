/* @name sqlCredentialUsed */
SELECT DISTINCT "StakeCredential".credential
FROM "StakeCredential"
INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
WHERE
  "StakeCredential".credential = ANY (:credentials)
  AND
  ("TxCredentialRelation".relation & (:relation)) > 0
  AND
  ("TxCredentialRelation".tx_id) <= (:until_tx_id)
  AND
  ("TxCredentialRelation".tx_id) > (:after_tx_id);