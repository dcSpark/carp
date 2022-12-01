/** Types generated for queries found in "app/models/dex/sqlDexSwap.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

export type stringArray = (string)[];

/** 'SqlDexSwap' parameters type */
export interface ISqlDexSwapParams {
  after_tx_id: string | null | void;
  asset_name1: BufferArray | null | void;
  asset_name2: BufferArray | null | void;
  dexes: stringArray | null | void;
  limit: string | null | void;
  policy_id1: BufferArray | null | void;
  policy_id2: BufferArray | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlDexSwap' return type */
export interface ISqlDexSwapResult {
  amount1: string;
  amount2: string;
  asset_name1: Buffer | null;
  asset_name2: Buffer | null;
  dex: string;
  operation: string;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
  tx_hash: Buffer;
}

/** 'SqlDexSwap' query type */
export interface ISqlDexSwapQuery {
  params: ISqlDexSwapParams;
  result: ISqlDexSwapResult;
}

const sqlDexSwapIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true,"dexes":true,"until_tx_id":true,"after_tx_id":true,"limit":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":370,"b":380}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":400,"b":411}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":431,"b":441}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":461,"b":472}]},{"name":"dexes","required":false,"transform":{"type":"scalar"},"locs":[{"a":1088,"b":1093}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":1701,"b":1712}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":1738,"b":1749}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":1790,"b":1795}]}],"statement":"WITH \"AssetPairs\" AS (\n  SELECT policy_id1, asset_name1, policy_id2, asset_name2\n  FROM\n    unnest(\n                                                                                                                                                                                                                                                                      \n      (:policy_id1)::bytea[],\n      (:asset_name1)::bytea[],\n      (:policy_id2)::bytea[],\n      (:asset_name2)::bytea[]\n    ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n)\nSELECT\n  \"Transaction\".hash AS tx_hash,  \n  \"Asset1\".policy_id AS \"policy_id1?\",\n  \"Asset1\".asset_name AS \"asset_name1?\",\n  \"Asset2\".policy_id AS \"policy_id2?\",\n  \"Asset2\".asset_name AS \"asset_name2?\",\n  \"Dex\".amount1,\n  \"Dex\".amount2,\n  \"Dex\".operation,\n  \"Dex\".dex\nFROM \"Dex\"\nJOIN \"Transaction\" ON \"Transaction\".id = \"Dex\".tx_id\nJOIN \"Address\" ON \"Address\".id = \"Dex\".address_id\nLEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".id = \"Dex\".asset1_id\nLEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".id = \"Dex\".asset2_id\nWHERE\n  \"Dex\".dex = ANY (:dexes)\n  AND\n  (\n    (\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea),\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    OR\n    (\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea),\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n  )\n  AND\n  \"Dex\".tx_id <= (:until_tx_id)\n  AND\n  \"Dex\".tx_id > (:after_tx_id)\nORDER BY \"Dex\".tx_id, \"Dex\".id\nLIMIT (:limit)"};

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
 *   "Transaction".hash AS tx_hash,  
 *   "Asset1".policy_id AS "policy_id1?",
 *   "Asset1".asset_name AS "asset_name1?",
 *   "Asset2".policy_id AS "policy_id2?",
 *   "Asset2".asset_name AS "asset_name2?",
 *   "Dex".amount1,
 *   "Dex".amount2,
 *   "Dex".operation,
 *   "Dex".dex
 * FROM "Dex"
 * JOIN "Transaction" ON "Transaction".id = "Dex".tx_id
 * JOIN "Address" ON "Address".id = "Dex".address_id
 * LEFT JOIN "NativeAsset" as "Asset1" ON "Asset1".id = "Dex".asset1_id
 * LEFT JOIN "NativeAsset" as "Asset2" ON "Asset2".id = "Dex".asset2_id
 * WHERE
 *   "Dex".dex = ANY (:dexes)
 *   AND
 *   (
 *     (
 *       COALESCE("Asset1".policy_id, ''::bytea),
 *       COALESCE("Asset1".asset_name, ''::bytea),
 *       COALESCE("Asset2".policy_id, ''::bytea),
 *       COALESCE("Asset2".asset_name, ''::bytea)
 *     ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *     OR
 *     (
 *       COALESCE("Asset2".policy_id, ''::bytea),
 *       COALESCE("Asset2".asset_name, ''::bytea),
 *       COALESCE("Asset1".policy_id, ''::bytea),
 *       COALESCE("Asset1".asset_name, ''::bytea)
 *     ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM "AssetPairs")
 *   )
 *   AND
 *   "Dex".tx_id <= (:until_tx_id)
 *   AND
 *   "Dex".tx_id > (:after_tx_id)
 * ORDER BY "Dex".tx_id, "Dex".id
 * LIMIT (:limit)
 * ```
 */
export const sqlDexSwap = new PreparedQuery<ISqlDexSwapParams,ISqlDexSwapResult>(sqlDexSwapIR);


