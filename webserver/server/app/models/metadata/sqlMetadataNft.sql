/* @name sqlMetadataNft */
WITH
  asset_pairs AS (
    SELECT policy_id, asset_name
    FROM
      unnest(
        (:policy_id)::bytea[],
        (:asset_name)::bytea[]
      ) x(policy_id,asset_name)
  ),
  native_assets AS (
    SELECT *
    FROM "NativeAsset"
    WHERE ("NativeAsset".policy_id, "NativeAsset".asset_name) in (SELECT policy_id, asset_name FROM asset_pairs)
  )
SELECT "Cip25Entry".payload, native_assets.policy_id, native_assets.asset_name
FROM
  native_assets
  INNER JOIN "TransactionMetadata"
    ON native_assets.first_tx = "TransactionMetadata".tx_id
  INNER JOIN "Cip25Entry"
    ON
      "Cip25Entry".asset_id = native_assets.id
      AND
      "Cip25Entry".metadata_id = "TransactionMetadata".id;