/** Types generated for queries found in "app/models/dex/sqlDexLastPriceMean.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlDexLastPriceMean' parameters type */
export interface ISqlDexLastPriceMeanParams {
  asset_name1: BufferArray | null | void;
  asset_name2: BufferArray | null | void;
  policy_id1: BufferArray | null | void;
  policy_id2: BufferArray | null | void;
}

/** 'SqlDexLastPriceMean' return type */
export interface ISqlDexLastPriceMeanResult {
  amount1: string | null;
  amount2: string | null;
  asset_name1: Buffer | null;
  asset_name2: Buffer | null;
  dex: string | null;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
}

/** 'SqlDexLastPriceMean' query type */
export interface ISqlDexLastPriceMeanQuery {
  params: ISqlDexLastPriceMeanParams;
  result: ISqlDexLastPriceMeanResult;
}

const sqlDexLastPriceMeanIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":370,"b":380}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":400,"b":411}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":431,"b":441}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":461,"b":472}]}],"statement":"WITH \"AssetPairs\" AS (\n  SELECT policy_id1, asset_name1, policy_id2, asset_name2\n  FROM\n    unnest(\n                                                                                                                                                                                                                                                                      \n      (:policy_id1)::bytea[],\n      (:asset_name1)::bytea[],\n      (:policy_id2)::bytea[],\n      (:asset_name2)::bytea[]\n    ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n)\nSELECT\n  DISTINCT ON(\"DexMeanPrice\".dex)\n\n  \"Asset1\".policy_id AS \"policy_id1?\",\n  \"Asset1\".asset_name AS \"asset_name1?\",\n  \"Asset2\".policy_id AS \"policy_id2?\",\n  \"Asset2\".asset_name AS \"asset_name2?\",\n  \"DexMeanPrice\".amount1,\n  \"DexMeanPrice\".amount2,\n  \"DexMeanPrice\".dex\nFROM \"DexMeanPrice\"\nLEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".id = \"DexMeanPrice\".asset1_id\nLEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".id = \"DexMeanPrice\".asset2_id\nWHERE\n  (\n    COALESCE(\"Asset1\".policy_id, ''::bytea),\n    COALESCE(\"Asset1\".asset_name, ''::bytea),\n    COALESCE(\"Asset2\".policy_id, ''::bytea),\n    COALESCE(\"Asset2\".asset_name, ''::bytea)\n  ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n  -- Add swap for another direction\n  OR\n  (\n    COALESCE(\"Asset2\".policy_id, ''::bytea),\n    COALESCE(\"Asset2\".asset_name, ''::bytea),\n    COALESCE(\"Asset1\".policy_id, ''::bytea),\n    COALESCE(\"Asset1\".asset_name, ''::bytea)\n  ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\nORDER BY \"DexMeanPrice\".dex, \"DexMeanPrice\".tx_id DESC, \"DexMeanPrice\".id DESC"};

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
 *   DISTINCT ON("DexMeanPrice".dex)
 * 
 *   "Asset1".policy_id AS "policy_id1?",
 *   "Asset1".asset_name AS "asset_name1?",
 *   "Asset2".policy_id AS "policy_id2?",
 *   "Asset2".asset_name AS "asset_name2?",
 *   "DexMeanPrice".amount1,
 *   "DexMeanPrice".amount2,
 *   "DexMeanPrice".dex
 * FROM "DexMeanPrice"
 * LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "DexMeanPrice".asset1_id
 * LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "DexMeanPrice".asset2_id
 * WHERE
 *   (
 *     COALESCE("Asset1".policy_id, ''::bytea),
 *     COALESCE("Asset1".asset_name, ''::bytea),
 *     COALESCE("Asset2".policy_id, ''::bytea),
 *     COALESCE("Asset2".asset_name, ''::bytea)
 *   ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *   -- Add swap for another direction
 *   OR
 *   (
 *     COALESCE("Asset2".policy_id, ''::bytea),
 *     COALESCE("Asset2".asset_name, ''::bytea),
 *     COALESCE("Asset1".policy_id, ''::bytea),
 *     COALESCE("Asset1".asset_name, ''::bytea)
 *   ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 * ORDER BY "DexMeanPrice".dex, "DexMeanPrice".tx_id DESC, "DexMeanPrice".id DESC
 * ```
 */
export const sqlDexLastPriceMean = new PreparedQuery<ISqlDexLastPriceMeanParams,ISqlDexLastPriceMeanResult>(sqlDexLastPriceMeanIR);


