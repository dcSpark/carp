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
  dex: string;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
}

/** 'SqlDexLastPrice' query type */
export interface ISqlDexLastPriceQuery {
  params: ISqlDexLastPriceParams;
  result: ISqlDexLastPriceResult;
}

const sqlDexLastPriceIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true,"operation1":true,"operation2":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":370,"b":380}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":400,"b":411}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":431,"b":441}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":461,"b":472}]},{"name":"operation1","required":false,"transform":{"type":"scalar"},"locs":[{"a":1248,"b":1258}]},{"name":"operation2","required":false,"transform":{"type":"scalar"},"locs":[{"a":1615,"b":1625}]}],"statement":"WITH \"AssetPairs\" AS (\n  SELECT policy_id1, asset_name1, policy_id2, asset_name2\n  FROM\n    unnest(\n                                                                                                                                                                                                                                                                      \n      (:policy_id1)::bytea[],\n      (:asset_name1)::bytea[],\n      (:policy_id2)::bytea[],\n      (:asset_name2)::bytea[]\n    ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n)\nSELECT\n  DISTINCT ON(\"Dex\".dex)\n\n  \"Asset1\".policy_id AS \"policy_id1?\",\n  \"Asset1\".asset_name AS \"asset_name1?\",\n  \"Asset2\".policy_id AS \"policy_id2?\",\n  \"Asset2\".asset_name AS \"asset_name2?\",\n  \"Dex\".amount1,\n  \"Dex\".amount2,\n  \"Dex\".dex\nFROM \"Dex\"\nLEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".id = \"Dex\".asset1_id\nLEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".id = \"Dex\".asset2_id\nWHERE\n  (\n    (\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea),\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    AND \"Dex\".operation = :operation1\n  )\n  -- Add swap for another direction\n  OR\n  (\n    (\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea),\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    AND \"Dex\".operation = :operation2\n  )\nORDER BY \"Dex\".dex, \"Dex\".tx_id DESC, \"Dex\".id DESC"};

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
 *   DISTINCT ON("Dex".dex)
 * 
 *   "Asset1".policy_id AS "policy_id1?",
 *   "Asset1".asset_name AS "asset_name1?",
 *   "Asset2".policy_id AS "policy_id2?",
 *   "Asset2".asset_name AS "asset_name2?",
 *   "Dex".amount1,
 *   "Dex".amount2,
 *   "Dex".dex
 * FROM "Dex"
 * LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "Dex".asset1_id
 * LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "Dex".asset2_id
 * WHERE
 *   (
 *     (
 *       COALESCE("Asset1".policy_id, ''::bytea),
 *       COALESCE("Asset1".asset_name, ''::bytea),
 *       COALESCE("Asset2".policy_id, ''::bytea),
 *       COALESCE("Asset2".asset_name, ''::bytea)
 *     ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *     AND "Dex".operation = :operation1
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
 *     AND "Dex".operation = :operation2
 *   )
 * ORDER BY "Dex".dex, "Dex".tx_id DESC, "Dex".id DESC
 * ```
 */
export const sqlDexLastPrice = new PreparedQuery<ISqlDexLastPriceParams,ISqlDexLastPriceResult>(sqlDexLastPriceIR);


