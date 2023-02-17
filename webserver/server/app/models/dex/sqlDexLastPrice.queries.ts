/** Types generated for queries found in "app/models/dex/sqlDexLastPrice.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlDexLastPrice' parameters type */
export interface ISqlDexLastPriceParams {
  asset_name1: BufferArray | null | void;
  asset_name2: BufferArray | null | void;
  operation1: string | null | void;
  operation2: string | null | void;
  policy_id1: BufferArray | null | void;
  policy_id2: BufferArray | null | void;
}

/** 'SqlDexLastPrice' return type */
export interface ISqlDexLastPriceResult {
  amount1: string;
  amount2: string;
  asset_name1: Buffer | null;
  asset_name2: Buffer | null;
  asset1_id: string | null;
  asset2_id: string | null;
  block_hash: Buffer;
  dex: string;
  epoch: number;
  height: number;
  id: string;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
  slot: number;
  tx_id: string;
}

/** 'SqlDexLastPrice' query type */
export interface ISqlDexLastPriceQuery {
  params: ISqlDexLastPriceParams;
  result: ISqlDexLastPriceResult;
}

const sqlDexLastPriceIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true,"operation1":true,"operation2":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":408,"b":418}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":442,"b":453}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":477,"b":487}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":511,"b":522}]},{"name":"operation1","required":false,"transform":{"type":"scalar"},"locs":[{"a":1708,"b":1718}]},{"name":"operation2","required":false,"transform":{"type":"scalar"},"locs":[{"a":2031,"b":2041}]}],"statement":"WITH\n  \"AssetPairs\" AS (\n      SELECT policy_id1, asset_name1, policy_id2, asset_name2\n      FROM\n        unnest(\n                                                                                                                                                                                                                                                                                          \n          (:policy_id1)::bytea[],\n          (:asset_name1)::bytea[],\n          (:policy_id2)::bytea[],\n          (:asset_name2)::bytea[]\n        ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n  ),\n  \"AssetIdPairs\" AS (\n        SELECT \"AssetPairs\".*, \"Asset1\".id as \"asset1_id\", \"Asset2\".id as \"asset2_id\"\n        FROM \"AssetPairs\"\n        LEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".policy_id = \"AssetPairs\".policy_id1 AND \"Asset1\".asset_name = \"AssetPairs\".asset_name1\n        LEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".policy_id = \"AssetPairs\".policy_id2 AND \"Asset2\".asset_name = \"AssetPairs\".asset_name2\n  ),\n  \"DexWithAssets\" AS (\n        SELECT\n        \"Asset1\".policy_id1 AS \"policy_id1?\",\n        \"Asset1\".asset_name1 AS \"asset_name1?\",\n        \"Asset2\".policy_id2 AS \"policy_id2?\",\n        \"Asset2\".asset_name2 AS \"asset_name2?\",\n        \"Dex\".asset1_id,\n        \"Dex\".asset2_id,\n        \"Dex\".amount1,\n        \"Dex\".amount2,\n        \"Dex\".dex,\n        \"Dex\".id,\n        \"Dex\".tx_id\n        FROM \"Dex\"\n        INNER JOIN \"AssetIdPairs\" as \"Asset1\"\n        ON\n              COALESCE(\"Dex\".asset1_id, -1) = COALESCE(\"Asset1\".asset1_id, -1) \n              AND\n              COALESCE(\"Dex\".asset2_id, -1) = COALESCE(\"Asset1\".asset2_id, -1)\n              AND\n              \"Dex\".operation = :operation1\n        -- Add swap for another direction\n        INNER JOIN \"AssetIdPairs\" as \"Asset2\"\n        ON\n              COALESCE(\"Dex\".asset2_id, -1) = COALESCE(\"Asset2\".asset2_id, -1)\n              AND\n              COALESCE(\"Dex\".asset1_id, -1) = COALESCE(\"Asset2\".asset1_id, -1)\n              AND \"Dex\".operation = :operation2\n  )\nSELECT\n      a.*,\n      \"Block\".hash as \"block_hash\",\n      \"Block\".height,\n      \"Block\".epoch,\n      \"Block\".slot\nFROM \"DexWithAssets\" a\nINNER JOIN (\n      SELECT\n      \"DexWithAssets\".dex, \"DexWithAssets\".asset1_id, \"DexWithAssets\".asset2_id,\n      MAX(\"DexWithAssets\".id) as \"id\"\n      FROM \"DexWithAssets\"\n      GROUP BY \"DexWithAssets\".dex, \"DexWithAssets\".asset1_id, \"DexWithAssets\".asset2_id\n) b ON a.id = b.id\nLEFT JOIN \"Transaction\" ON \"Transaction\".id = a.tx_id\nLEFT JOIN \"Block\" ON \"Block\".id = \"Transaction\".block_id"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   "AssetPairs" AS (
 *       SELECT policy_id1, asset_name1, policy_id2, asset_name2
 *       FROM
 *         unnest(
 *                                                                                                                                                                                                                                                                                           
 *           (:policy_id1)::bytea[],
 *           (:asset_name1)::bytea[],
 *           (:policy_id2)::bytea[],
 *           (:asset_name2)::bytea[]
 *         ) x(policy_id1, asset_name1, policy_id2, asset_name2)
 *   ),
 *   "AssetIdPairs" AS (
 *         SELECT "AssetPairs".*, "Asset1".id as "asset1_id", "Asset2".id as "asset2_id"
 *         FROM "AssetPairs"
 *         LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".policy_id = "AssetPairs".policy_id1 AND "Asset1".asset_name = "AssetPairs".asset_name1
 *         LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".policy_id = "AssetPairs".policy_id2 AND "Asset2".asset_name = "AssetPairs".asset_name2
 *   ),
 *   "DexWithAssets" AS (
 *         SELECT
 *         "Asset1".policy_id1 AS "policy_id1?",
 *         "Asset1".asset_name1 AS "asset_name1?",
 *         "Asset2".policy_id2 AS "policy_id2?",
 *         "Asset2".asset_name2 AS "asset_name2?",
 *         "Dex".asset1_id,
 *         "Dex".asset2_id,
 *         "Dex".amount1,
 *         "Dex".amount2,
 *         "Dex".dex,
 *         "Dex".id,
 *         "Dex".tx_id
 *         FROM "Dex"
 *         INNER JOIN "AssetIdPairs" as "Asset1"
 *         ON
 *               COALESCE("Dex".asset1_id, -1) = COALESCE("Asset1".asset1_id, -1) 
 *               AND
 *               COALESCE("Dex".asset2_id, -1) = COALESCE("Asset1".asset2_id, -1)
 *               AND
 *               "Dex".operation = :operation1
 *         -- Add swap for another direction
 *         INNER JOIN "AssetIdPairs" as "Asset2"
 *         ON
 *               COALESCE("Dex".asset2_id, -1) = COALESCE("Asset2".asset2_id, -1)
 *               AND
 *               COALESCE("Dex".asset1_id, -1) = COALESCE("Asset2".asset1_id, -1)
 *               AND "Dex".operation = :operation2
 *   )
 * SELECT
 *       a.*,
 *       "Block".hash as "block_hash",
 *       "Block".height,
 *       "Block".epoch,
 *       "Block".slot
 * FROM "DexWithAssets" a
 * INNER JOIN (
 *       SELECT
 *       "DexWithAssets".dex, "DexWithAssets".asset1_id, "DexWithAssets".asset2_id,
 *       MAX("DexWithAssets".id) as "id"
 *       FROM "DexWithAssets"
 *       GROUP BY "DexWithAssets".dex, "DexWithAssets".asset1_id, "DexWithAssets".asset2_id
 * ) b ON a.id = b.id
 * LEFT JOIN "Transaction" ON "Transaction".id = a.tx_id
 * LEFT JOIN "Block" ON "Block".id = "Transaction".block_id
 * ```
 */
export const sqlDexLastPrice = new PreparedQuery<ISqlDexLastPriceParams,ISqlDexLastPriceResult>(sqlDexLastPriceIR);


