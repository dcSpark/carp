/** Types generated for queries found in "app/models/transaction/sqlHistoryForCredentials.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlHistoryForCredentials' parameters type */
export interface ISqlHistoryForCredentialsParams {
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  limit: string | null | void;
  relation: number | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlHistoryForCredentials' return type */
export interface ISqlHistoryForCredentialsResult {
  block_hash: Buffer;
  epoch: number;
  era: number;
  hash: Buffer;
  height: number;
  id: string;
  is_valid: boolean;
  payload: Buffer;
  slot: number;
  tx_index: number;
}

/** 'SqlHistoryForCredentials' query type */
export interface ISqlHistoryForCredentialsQuery {
  params: ISqlHistoryForCredentialsParams;
  result: ISqlHistoryForCredentialsResult;
}

const sqlHistoryForCredentialsIR: any = {"usedParamSet":{"credentials":true,"relation":true,"until_tx_id":true,"after_tx_id":true,"limit":true},"params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"locs":[{"a":288,"b":299}]},{"name":"relation","required":false,"transform":{"type":"scalar"},"locs":[{"a":354,"b":362}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":464,"b":475}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":527,"b":538}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":598,"b":603}]}],"statement":"WITH\n  tx_relations AS (\n    SELECT DISTINCT ON (\"TxCredentialRelation\".tx_id) \"TxCredentialRelation\".tx_id\n    FROM \"StakeCredential\"\n    INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n    WHERE\n      \"StakeCredential\".credential = ANY (:credentials)\n      AND\n      (\"TxCredentialRelation\".relation & (:relation)) > 0\n      AND\n                                            \n      \"TxCredentialRelation\".tx_id <= (:until_tx_id)\n      AND \n      \"TxCredentialRelation\".tx_id > (:after_tx_id)\n    ORDER BY \"TxCredentialRelation\".tx_id ASC\n    LIMIT (:limit)\n  )\nSELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\nFROM tx_relations\nINNER JOIN \"Transaction\" ON tx_relations.tx_id = \"Transaction\".id\nINNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   tx_relations AS (
 *     SELECT DISTINCT ON ("TxCredentialRelation".tx_id) "TxCredentialRelation".tx_id
 *     FROM "StakeCredential"
 *     INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *     WHERE
 *       "StakeCredential".credential = ANY (:credentials)
 *       AND
 *       ("TxCredentialRelation".relation & (:relation)) > 0
 *       AND
 *                                             
 *       "TxCredentialRelation".tx_id <= (:until_tx_id)
 *       AND 
 *       "TxCredentialRelation".tx_id > (:after_tx_id)
 *     ORDER BY "TxCredentialRelation".tx_id ASC
 *     LIMIT (:limit)
 *   )
 * SELECT "Transaction".id,
 *         "Transaction".payload,
 *         "Transaction".hash,
 *         "Transaction".tx_index,
 *         "Transaction".is_valid,
 *         "Block".hash AS block_hash,
 *         "Block".epoch,
 *         "Block".slot,
 *         "Block".era,
 *         "Block".height
 * FROM tx_relations
 * INNER JOIN "Transaction" ON tx_relations.tx_id = "Transaction".id
 * INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 * ```
 */
export const sqlHistoryForCredentials = new PreparedQuery<ISqlHistoryForCredentialsParams,ISqlHistoryForCredentialsResult>(sqlHistoryForCredentialsIR);


