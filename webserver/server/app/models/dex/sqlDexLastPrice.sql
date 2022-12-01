/* @name sqlDexLastPrice */
WITH "AssetPairs" AS (
  SELECT policy_id1, asset_name1, policy_id2, asset_name2
  FROM
    unnest(
      /*
      Aparrently, we can't make pgtyped understand that these are actually (bytea | NULL)[].
      We will pass in ('', '') instead of (NULL, NULL) for ADA and do the NULL->'' conversion
      below when filtering the assets (see the COALESCE).
      */
      (:policy_id1)::bytea[],
      (:asset_name1)::bytea[],
      (:policy_id2)::bytea[],
      (:asset_name2)::bytea[]
    ) x(policy_id1, asset_name1, policy_id2, asset_name2)
)
SELECT
  DISTINCT ON("Dex".dex)

  "Asset1".policy_id AS "policy_id1?",
  "Asset1".asset_name AS "asset_name1?",
  "Asset2".policy_id AS "policy_id2?",
  "Asset2".asset_name AS "asset_name2?",
  "Dex".amount1,
  "Dex".amount2,
  "Dex".dex
FROM "Dex"
LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "Dex".asset1_id
LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "Dex".asset2_id
WHERE
  (
    (
      COALESCE("Asset1".policy_id, ''::bytea),
      COALESCE("Asset1".asset_name, ''::bytea),
      COALESCE("Asset2".policy_id, ''::bytea),
      COALESCE("Asset2".asset_name, ''::bytea)
    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
    AND "Dex".operation = :operation1
  )
  -- Add swap for another direction
  OR
  (
    (
      COALESCE("Asset2".policy_id, ''::bytea),
      COALESCE("Asset2".asset_name, ''::bytea),
      COALESCE("Asset1".policy_id, ''::bytea),
      COALESCE("Asset1".asset_name, ''::bytea)
    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
    AND "Dex".operation = :operation2
  )
ORDER BY "Dex".dex, "Dex".tx_id DESC, "Dex".id DESC;
