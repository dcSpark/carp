/** Types generated for queries found in "app/models/dex/sqlDexMeanPrice.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type BufferArray = (Buffer)[];

export type NumberOrString = number | string;

export type NumberOrStringArray = (NumberOrString)[];

/** 'SqlDexMeanPrice' parameters type */
export interface ISqlDexMeanPriceParams {
  after_tx_id?: NumberOrString | null | void;
  asset_name1?: BufferArray | null | void;
  asset_name2?: BufferArray | null | void;
  dexes?: NumberOrStringArray | null | void;
  limit?: NumberOrString | null | void;
  policy_id1?: BufferArray | null | void;
  policy_id2?: BufferArray | null | void;
  until_tx_id?: NumberOrString | null | void;
}

/** 'SqlDexMeanPrice' return type */
export interface ISqlDexMeanPriceResult {
  amount1: string;
  amount2: string;
  asset_name1: Buffer | null;
  asset_name2: Buffer | null;
  dex: string;
  policy_id1: Buffer | null;
  policy_id2: Buffer | null;
  tx_hash: Buffer;
}

/** 'SqlDexMeanPrice' query type */
export interface ISqlDexMeanPriceQuery {
  params: ISqlDexMeanPriceParams;
  result: ISqlDexMeanPriceResult;
}

const sqlDexMeanPriceIR: any = {"usedParamSet":{"policy_id1":true,"asset_name1":true,"policy_id2":true,"asset_name2":true,"dexes":true,"until_tx_id":true,"after_tx_id":true,"limit":true},"params":[{"name":"policy_id1","required":false,"transform":{"type":"scalar"},"locs":[{"a":370,"b":380}]},{"name":"asset_name1","required":false,"transform":{"type":"scalar"},"locs":[{"a":400,"b":411}]},{"name":"policy_id2","required":false,"transform":{"type":"scalar"},"locs":[{"a":431,"b":441}]},{"name":"asset_name2","required":false,"transform":{"type":"scalar"},"locs":[{"a":461,"b":472}]},{"name":"dexes","required":false,"transform":{"type":"scalar"},"locs":[{"a":1024,"b":1029}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":1665,"b":1676}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":1702,"b":1713}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":1754,"b":1759}]}],"statement":"WITH \"AssetPairs\" AS (\n  SELECT policy_id1, asset_name1, policy_id2, asset_name2\n  FROM\n    unnest(\n                                                                                                                                                                                                                                                                      \n      (:policy_id1)::bytea[],\n      (:asset_name1)::bytea[],\n      (:policy_id2)::bytea[],\n      (:asset_name2)::bytea[]\n    ) x(policy_id1, asset_name1, policy_id2, asset_name2)\n)\nSELECT\n  \"Transaction\".hash AS tx_hash,\n  \"Dex\".dex as dex,\n  \"Asset1\".policy_id AS \"policy_id1?\",\n  \"Asset1\".asset_name AS \"asset_name1?\",\n  \"Asset2\".policy_id AS \"policy_id2?\",\n  \"Asset2\".asset_name AS \"asset_name2?\",\n  \"Dex\".amount1,\n  \"Dex\".amount2\nFROM \"Dex\"\nJOIN \"Transaction\" ON \"Transaction\".id = \"Dex\".tx_id\nLEFT JOIN \"NativeAsset\" as \"Asset1\" ON \"Asset1\".id = \"Dex\".asset1_id\nLEFT JOIN \"NativeAsset\" as \"Asset2\" ON \"Asset2\".id = \"Dex\".asset2_id\nWHERE\n  \"Dex\".dex = ANY (:dexes)\n  AND\n  (\n    (\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea),\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n    OR\n    (\n      COALESCE(\"Asset2\".policy_id, ''::bytea),\n      COALESCE(\"Asset2\".asset_name, ''::bytea),\n      COALESCE(\"Asset1\".policy_id, ''::bytea),\n      COALESCE(\"Asset1\".asset_name, ''::bytea)\n    ) IN (SELECT policy_id1, asset_name1, policy_id2, asset_name2 FROM \"AssetPairs\")\n  )\n  AND\n  \"Dex\".operation = 2\n  AND\n  \"Dex\".tx_id <= (:until_tx_id)\n  AND\n  \"Dex\".tx_id > (:after_tx_id)\nORDER BY \"Dex\".tx_id, \"Dex\".id\nLIMIT (:limit)"};

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
 *   "Dex".dex as dex,
 *   "Asset1".policy_id AS "policy_id1?",
 *   "Asset1".asset_name AS "asset_name1?",
 *   "Asset2".policy_id AS "policy_id2?",
 *   "Asset2".asset_name AS "asset_name2?",
 *   "Dex".amount1,
 *   "Dex".amount2
 * FROM "Dex"
 * JOIN "Transaction" ON "Transaction".id = "Dex".tx_id
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
 *   "Dex".operation = 2
 *   AND
 *   "Dex".tx_id <= (:until_tx_id)
 *   AND
 *   "Dex".tx_id > (:after_tx_id)
 * ORDER BY "Dex".tx_id, "Dex".id
 * LIMIT (:limit)
 * ```
 */
export const sqlDexMeanPrice = new PreparedQuery<ISqlDexMeanPriceParams,ISqlDexMeanPriceResult>(sqlDexMeanPriceIR);


