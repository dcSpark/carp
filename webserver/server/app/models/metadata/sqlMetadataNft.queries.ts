/** Types generated for queries found in "app/models/metadata/sqlMetadataNft.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlMetadataNft' parameters type */
export interface ISqlMetadataNftParams {
  asset_name: BufferArray | null | void;
  policy_id: BufferArray | null | void;
}

/** 'SqlMetadataNft' return type */
export interface ISqlMetadataNftResult {
  asset_name: Buffer;
  payload: Buffer;
  policy_id: Buffer;
}

/** 'SqlMetadataNft' query type */
export interface ISqlMetadataNftQuery {
  params: ISqlMetadataNftParams;
  result: ISqlMetadataNftResult;
}

const sqlMetadataNftIR: any = {"name":"sqlMetadataNft","params":[{"name":"policy_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":117,"b":125,"line":7,"col":10}]}},{"name":"asset_name","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":148,"b":157,"line":8,"col":10}]}}],"usedParamSet":{"policy_id":true,"asset_name":true},"statement":{"body":"WITH\n  asset_pairs AS (\n    SELECT policy_id, asset_name\n    FROM\n      unnest(\n        (:policy_id)::bytea[],\n        (:asset_name)::bytea[]\n      ) x(policy_id,asset_name)\n  ),\n  native_assets AS (\n    SELECT *\n    FROM \"NativeAsset\"\n    WHERE (\"NativeAsset\".policy_id, \"NativeAsset\".asset_name) in (SELECT policy_id, asset_name FROM asset_pairs)\n  )\nSELECT \"Cip25Entry\".payload, native_assets.policy_id, native_assets.asset_name\nFROM\n  native_assets\n  INNER JOIN \"TransactionMetadata\"\n    ON native_assets.first_tx = \"TransactionMetadata\".tx_id\n  INNER JOIN \"Cip25Entry\"\n    ON\n      \"Cip25Entry\".asset_id = native_assets.id\n      AND\n      \"Cip25Entry\".metadata_id = \"TransactionMetadata\".id","loc":{"a":27,"b":721,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   asset_pairs AS (
 *     SELECT policy_id, asset_name
 *     FROM
 *       unnest(
 *         (:policy_id)::bytea[],
 *         (:asset_name)::bytea[]
 *       ) x(policy_id,asset_name)
 *   ),
 *   native_assets AS (
 *     SELECT *
 *     FROM "NativeAsset"
 *     WHERE ("NativeAsset".policy_id, "NativeAsset".asset_name) in (SELECT policy_id, asset_name FROM asset_pairs)
 *   )
 * SELECT "Cip25Entry".payload, native_assets.policy_id, native_assets.asset_name
 * FROM
 *   native_assets
 *   INNER JOIN "TransactionMetadata"
 *     ON native_assets.first_tx = "TransactionMetadata".tx_id
 *   INNER JOIN "Cip25Entry"
 *     ON
 *       "Cip25Entry".asset_id = native_assets.id
 *       AND
 *       "Cip25Entry".metadata_id = "TransactionMetadata".id
 * ```
 */
export const sqlMetadataNft = new PreparedQuery<ISqlMetadataNftParams,ISqlMetadataNftResult>(sqlMetadataNftIR);


