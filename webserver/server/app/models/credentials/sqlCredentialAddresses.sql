/* @name sqlCredentialAddresses */
WITH
  max_address_id AS (
    SELECT "Address".id
    FROM "Address"
    WHERE "Address".first_tx <= (:until_tx_id)
    ORDER BY "Address".first_tx DESC
    LIMIT 1
  ),
  min_address_id AS (
    SELECT
      CASE
            WHEN (:after_address)::bytea IS NULL then -1
            WHEN (:after_address)::bytea IS NOT NULL then (
              SELECT "Address".id
              FROM "Address"
              WHERE "Address".payload = (:after_address)::bytea
            )
      END
  ),
  relations AS (
    SELECT "AddressCredentialRelation".address_id
    FROM "StakeCredential"
    INNER JOIN "AddressCredentialRelation" ON "StakeCredential".id = "AddressCredentialRelation".credential_id
    WHERE
      "StakeCredential".credential = ANY (:credentials)
      AND
      "AddressCredentialRelation".address_id > (SELECT * FROM min_address_id)
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
