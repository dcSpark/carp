/* @name sqlCredentialUsed */
SELECT DISTINCT "StakeCredential".credential
FROM "StakeCredential"
WHERE
  "StakeCredential".credential = ANY (:credentials)
  AND
  ("StakeCredential".first_tx) <= (:until_tx_id)
  AND
  ("StakeCredential".first_tx) > (:after_tx_id);