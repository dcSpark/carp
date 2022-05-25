/* @name sqlCredentialAddresses */
WITH
  max_address_id AS (
    SELECT MAX("Address".id)
    FROM "Address"
    WHERE "Address".first_tx <= (:until_tx_id)
  ),
  min_address_id AS (
    SELECT MIN("Address".id)
    FROM "Address"
    WHERE "Address".first_tx > (:after_tx_id)
  ),
  relations AS (
    SELECT "AddressCredentialRelation".address_id
    FROM "StakeCredential"
    INNER JOIN "AddressCredentialRelation" ON "StakeCredential".id = "AddressCredentialRelation".credential_id
    WHERE
      "StakeCredential".credential = ANY (:credentials)
      AND
      "AddressCredentialRelation".address_id >= (SELECT * FROM min_address_id)
      AND
      "AddressCredentialRelation".address_id <= (SELECT * FROM max_address_id)
      ORDER BY "AddressCredentialRelation".address_id ASC
      LIMIT (:double_limit)
  )
SELECT DISTINCT ON ("Address".id) "Address".payload, "Address".first_tx
FROM "Address"
WHERE "Address".id in (SELECT * FROM relations)
ORDER BY "Address".id ASC
LIMIT (:limit);
