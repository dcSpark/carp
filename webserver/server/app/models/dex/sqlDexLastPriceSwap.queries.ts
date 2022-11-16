/** Types generated for queries found in "app/models/dex/sqlDexLastPriceSwap.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlDexLastPriceSwap' parameters type */
export interface ISqlDexLastPriceSwapParams {
  asset_name1: BufferArray | null | void;
  asset_name2: BufferArray | null | void;
  direction: boolean | null | void;
  policy_id1: BufferArray | null | void;
  policy_id2: BufferArray | null | void;
}

/** 'SqlDexLastPriceSwap' return type */
export interface ISqlDexLastPriceSwapResult {
  amount1: string;
  amount2: string;
  asset_name1: Buffer | null;
  asset_name2: Buffer | null;
  dex: string;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
}

/** 'SqlDexLastPriceSwap' query type */
export interface ISqlDexLastPriceSwapQuery {
  params: ISqlDexLastPriceSwapParams;
  result: ISqlDexLastPriceSwapResult;
}

const sqlDexLastPriceSwapIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true,"direction":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":370,"b":380}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":400,"b":411}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":431,"b":441}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":461,"b":472}]},{"name":"direction","required":false,"transform":{"type":"scalar"},"locs":[{"a":1287,"b":1296},{"a":1658,"b":1667}]}],"statement":"WITH \"AssetPairs\" AS (\n  SELECT policy_id1, asset_name1, policy_id2, asset_name2\n  FROM\n    unnest(\n                                                                                                                                                                                                                                                                      \n      (:policy_id1)::bytea[],\n      (:asset_name1)::bytea[],\n      (:policy_id2)::bytea[],\n      (:asset_name2)::bytea[]\n    ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n)\nSELECT\n  DISTINCT ON(\"DexSwap\".address_id)\n\n  \"Asset1\".policy_id AS \"policy_id1?\",\n  \"Asset1\".asset_name AS \"asset_name1?\",\n  \"Asset2\".policy_id AS \"policy_id2?\",\n  \"Asset2\".asset_name AS \"asset_name2?\",\n  \"DexSwap\".amount1,\n  \"DexSwap\".amount2,\n  \"DexSwap\".dex\nFROM \"DexSwap\"\nLEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".id = \"DexSwap\".asset1_id\nLEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".id = \"DexSwap\".asset2_id\nWHERE\n  (\n    (\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea),\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    AND \"DexSwap\".direction = :direction\n  )\n  -- Add swap for another direction\n  OR\n  (\n    (\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea),\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    AND \"DexSwap\".direction != :direction\n  )\nORDER BY \"DexSwap\".address_id, \"DexSwap\".tx_id DESC, \"DexSwap\".id DESC"};

/**
 * Query generated from SQL:
 * ```
 * WITH "AssetPairs" AS (
 *   SELECT policy_id1, asset_name1, policy_id2, asset_name2
 *   FROM
 *     unnest(
 *                                                                                                                                                                                                                                                                       
 *       (:policy_id1)::bytea[],
 *       (:asset_name1)::bytea[],
 *       (:policy_id2)::bytea[],
 *       (:asset_name2)::bytea[]
 *     ) x(policy_id1, asset_name1, policy_id2, asset_name2)
 * )
 * SELECT
 *   DISTINCT ON("DexSwap".address_id)
 * 
 *   "Asset1".policy_id AS "policy_id1?",
 *   "Asset1".asset_name AS "asset_name1?",
 *   "Asset2".policy_id AS "policy_id2?",
 *   "Asset2".asset_name AS "asset_name2?",
 *   "DexSwap".amount1,
 *   "DexSwap".amount2,
 *   "DexSwap".dex
 * FROM "DexSwap"
 * LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "DexSwap".asset1_id
 * LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "DexSwap".asset2_id
 * WHERE
 *   (
 *     (
 *       COALESCE("Asset1".policy_id, ''::bytea),
 *       COALESCE("Asset1".asset_name, ''::bytea),
 *       COALESCE("Asset2".policy_id, ''::bytea),
 *       COALESCE("Asset2".asset_name, ''::bytea)
 *     ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *     AND "DexSwap".direction = :direction
 *   )
 *   -- Add swap for another direction
 *   OR
 *   (
 *     (
 *       COALESCE("Asset2".policy_id, ''::bytea),
 *       COALESCE("Asset2".asset_name, ''::bytea),
 *       COALESCE("Asset1".policy_id, ''::bytea),
 *       COALESCE("Asset1".asset_name, ''::bytea)
 *     ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *     AND "DexSwap".direction != :direction
 *   )
 * ORDER BY "DexSwap".address_id, "DexSwap".tx_id DESC, "DexSwap".id DESC
 * ```
 */
export const sqlDexLastPriceSwap = new PreparedQuery<ISqlDexLastPriceSwapParams,ISqlDexLastPriceSwapResult>(sqlDexLastPriceSwapIR);


