/* @name sqlAddressUsed */
SELECT DISTINCT "Address".payload
FROM "Address"
WHERE
  "Address".payload = ANY (:addresses)
  AND
  ("Address".first_tx) <= (:until_tx_id)
  AND
  ("Address".first_tx) > (:after_tx_id);
