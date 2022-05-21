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
SELECT "TransactionMetadata".payload, native_assets.policy_id, native_assets.asset_name
FROM
  (
    SELECT "AssetMint".asset_id, MIN("AssetMint".tx_id) as tx_id
    FROM "AssetMint"
    INNER JOIN native_assets ON native_assets.id = "AssetMint".asset_id
    GROUP BY "AssetMint".asset_id
  ) asset_and_tx
  INNER JOIN native_assets
    ON
      native_assets.id = asset_and_tx.asset_id
  INNER JOIN "Cip25Entry"
    ON
      "Cip25Entry".native_asset_id = asset_and_tx.asset_id
      AND
      "Cip25Entry".tx_id = asset_and_tx.tx_id
  INNER JOIN "TransactionMetadata"
    ON
      "Cip25Entry".tx_id = "TransactionMetadata".tx_id
      AND
      "Cip25Entry".label = "TransactionMetadata".label;