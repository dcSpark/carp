/* @name sqlDexLastPrice */
WITH
  "AssetPairs" AS (
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
  ),
  "AssetIdPairs" AS (
        SELECT "AssetPairs".*, "Asset1".id as "asset1_id", "Asset2".id as "asset2_id"
        FROM "AssetPairs"
        LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".policy_id = "AssetPairs".policy_id1 AND "Asset1".asset_name = "AssetPairs".asset_name1
        LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".policy_id = "AssetPairs".policy_id2 AND "Asset2".asset_name = "AssetPairs".asset_name2
  ),
  "DexWithAssets" AS (
        SELECT
        "Asset1".policy_id1 AS "policy_id1?",
        "Asset1".asset_name1 AS "asset_name1?",
        "Asset2".policy_id2 AS "policy_id2?",
        "Asset2".asset_name2 AS "asset_name2?",
        "Dex".asset1_id,
        "Dex".asset2_id,
        "Dex".amount1,
        "Dex".amount2,
        "Dex".dex,
        "Dex".id,
        "Dex".tx_id
        FROM "Dex"
        INNER JOIN "AssetIdPairs" as "Asset1"
        ON
              COALESCE("Dex".asset1_id, -1) = COALESCE("Asset1".asset1_id, -1) 
              AND
              COALESCE("Dex".asset2_id, -1) = COALESCE("Asset1".asset2_id, -1)
              AND
              "Dex".operation = :operation1
        -- Add swap for another direction
        INNER JOIN "AssetIdPairs" as "Asset2"
        ON
              COALESCE("Dex".asset2_id, -1) = COALESCE("Asset2".asset2_id, -1)
              AND
              COALESCE("Dex".asset1_id, -1) = COALESCE("Asset2".asset1_id, -1)
              AND "Dex".operation = :operation2
  )
SELECT
      a.*,
      "Block".hash as "block_hash",
      "Block".height,
      "Block".epoch,
      "Block".slot
FROM "DexWithAssets" a
INNER JOIN (
      SELECT
      "DexWithAssets".dex, "DexWithAssets".asset1_id, "DexWithAssets".asset2_id,
      MAX("DexWithAssets".id) as "id"
      FROM "DexWithAssets"
      GROUP BY "DexWithAssets".dex, "DexWithAssets".asset1_id, "DexWithAssets".asset2_id
) b ON a.id = b.id
LEFT JOIN "Transaction" ON "Transaction".id = a.tx_id
LEFT JOIN "Block" ON "Block".id = "Transaction".block_id;
