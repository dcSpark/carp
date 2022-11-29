/* @name sqlDexLastPriceSwap */
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
  DISTINCT ON("DexSwap".dex)

  "Asset1".policy_id AS "policy_id1?",
  "Asset1".asset_name AS "asset_name1?",
  "Asset2".policy_id AS "policy_id2?",
  "Asset2".asset_name AS "asset_name2?",
  "DexSwap".amount1,
  "DexSwap".amount2,
  "DexSwap".dex
FROM "DexSwap"
LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "DexSwap".asset1_id
LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "DexSwap".asset2_id
WHERE
  (
    (
      COALESCE("Asset1".policy_id, ''::bytea),
      COALESCE("Asset1".asset_name, ''::bytea),
      COALESCE("Asset2".policy_id, ''::bytea),
      COALESCE("Asset2".asset_name, ''::bytea)
    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
    AND "DexSwap".operation = :operation
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
    AND "DexSwap".operation != :operation
  )
ORDER BY "DexSwap".dex, "DexSwap".tx_id DESC, "DexSwap".id DESC;
