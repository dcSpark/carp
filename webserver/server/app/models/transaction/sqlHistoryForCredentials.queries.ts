/** Types generated for queries found in "app/models/transaction/sqlHistoryForCredentials.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type BufferArray = (Buffer)[];

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'SqlHistoryForCredentials' parameters type */
export interface ISqlHistoryForCredentialsParams {
  after_tx_id?: NumberOrString | null | void;
  credentials?: BufferArray | null | void;
  limit?: NumberOrString | null | void;
  relation?: number | null | void;
  until_tx_id?: NumberOrString | null | void;
  with_input_context: boolean;
}

/** 'SqlHistoryForCredentials' return type */
export interface ISqlHistoryForCredentialsResult {
  block_hash: Buffer;
  epoch: number;
  era: number;
  hash: Buffer;
  height: number;
  id: string | null;
  input_addresses: Json | null;
  is_valid: boolean;
  metadata: Buffer | null;
  payload: Buffer;
  slot: number;
  tx_index: number;
}

/** 'SqlHistoryForCredentials' query type */
export interface ISqlHistoryForCredentialsQuery {
  params: ISqlHistoryForCredentialsParams;
  result: ISqlHistoryForCredentialsResult;
}

const sqlHistoryForCredentialsIR: any = {"usedParamSet":{"credentials":true,"relation":true,"until_tx_id":true,"after_tx_id":true,"limit":true,"with_input_context":true},"params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"locs":[{"a":288,"b":299}]},{"name":"relation","required":false,"transform":{"type":"scalar"},"locs":[{"a":354,"b":362}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":464,"b":475}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":527,"b":538}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":598,"b":603}]},{"name":"with_input_context","required":true,"transform":{"type":"scalar"},"locs":[{"a":2446,"b":2465},{"a":2529,"b":2548}]}],"statement":"WITH\n  tx_relations AS (\n    SELECT DISTINCT ON (\"TxCredentialRelation\".tx_id) \"TxCredentialRelation\".tx_id\n    FROM \"StakeCredential\"\n    INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n    WHERE\n      \"StakeCredential\".credential = ANY (:credentials)\n      AND\n      (\"TxCredentialRelation\".relation & (:relation)) > 0\n      AND\n                                            \n      \"TxCredentialRelation\".tx_id <= (:until_tx_id)\n      AND \n      \"TxCredentialRelation\".tx_id > (:after_tx_id)\n    ORDER BY \"TxCredentialRelation\".tx_id ASC\n    LIMIT (:limit)\n  ),\n  base_query AS (\n        SELECT \"Transaction\".id,\n            \"Transaction\".payload as \"payload!\",\n            \"Transaction\".hash as \"hash!\",\n            \"Transaction\".tx_index as \"tx_index!\",\n            \"Transaction\".is_valid as \"is_valid!\",\n            \"Block\".hash AS \"block_hash!\",\n            \"Block\".epoch as \"epoch!\",\n            \"Block\".slot as \"slot!\",\n            \"Block\".era as \"era!\",\n            \"Block\".height as \"height!\",\n            NULL :: bytea as metadata,\n            NULL :: json as input_addresses\n    FROM tx_relations\n    INNER JOIN \"Transaction\" ON tx_relations.tx_id = \"Transaction\".id\n    INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n  ),\n  query_with_inputs_and_metadata AS (\n        SELECT \"Transaction\".id,\n                \"Transaction\".payload as \"payload!\",\n                \"Transaction\".hash as \"hash!\",\n                \"Transaction\".tx_index as \"tx_index!\",\n                \"Transaction\".is_valid as \"is_valid!\",\n                \"Block\".hash as \"block_hash!\",\n                \"Block\".epoch as \"epoch!\",\n                \"Block\".slot as \"slot!\",\n                \"Block\".era as \"era!\",\n                \"Block\".height as \"height!\",\n                \"TransactionMetadata\".payload AS metadata,\n                json_agg(DISTINCT \"Address\".PAYLOAD) input_addresses\n        FROM tx_relations\n        INNER JOIN \"Transaction\" ON tx_relations.tx_id = \"Transaction\".id\n        INNER JOIN \"TransactionInput\" ON \"TransactionInput\".tx_id = \"Transaction\".id\n        INNER JOIN \"Address\" ON \"Address\".id = \"TransactionInput\".address_id\n        LEFT JOIN \"TransactionMetadata\" ON \"Transaction\".id = \"TransactionMetadata\".tx_id\n        INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n        GROUP BY \"Transaction\".id, \"Block\".id, \"TransactionMetadata\".id\n  )\nSELECT * FROM base_query WHERE NOT :with_input_context!\nUNION ALL (SELECT * FROM query_with_inputs_and_metadata WHERE :with_input_context!)"};

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
 *   ),
 *   base_query AS (
 *         SELECT "Transaction".id,
 *             "Transaction".payload as "payload!",
 *             "Transaction".hash as "hash!",
 *             "Transaction".tx_index as "tx_index!",
 *             "Transaction".is_valid as "is_valid!",
 *             "Block".hash AS "block_hash!",
 *             "Block".epoch as "epoch!",
 *             "Block".slot as "slot!",
 *             "Block".era as "era!",
 *             "Block".height as "height!",
 *             NULL :: bytea as metadata,
 *             NULL :: json as input_addresses
 *     FROM tx_relations
 *     INNER JOIN "Transaction" ON tx_relations.tx_id = "Transaction".id
 *     INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *   ),
 *   query_with_inputs_and_metadata AS (
 *         SELECT "Transaction".id,
 *                 "Transaction".payload as "payload!",
 *                 "Transaction".hash as "hash!",
 *                 "Transaction".tx_index as "tx_index!",
 *                 "Transaction".is_valid as "is_valid!",
 *                 "Block".hash as "block_hash!",
 *                 "Block".epoch as "epoch!",
 *                 "Block".slot as "slot!",
 *                 "Block".era as "era!",
 *                 "Block".height as "height!",
 *                 "TransactionMetadata".payload AS metadata,
 *                 json_agg(DISTINCT "Address".PAYLOAD) input_addresses
 *         FROM tx_relations
 *         INNER JOIN "Transaction" ON tx_relations.tx_id = "Transaction".id
 *         INNER JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
 *         INNER JOIN "Address" ON "Address".id = "TransactionInput".address_id
 *         LEFT JOIN "TransactionMetadata" ON "Transaction".id = "TransactionMetadata".tx_id
 *         INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *         GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
 *   )
 * SELECT * FROM base_query WHERE NOT :with_input_context!
 * UNION ALL (SELECT * FROM query_with_inputs_and_metadata WHERE :with_input_context!)
 * ```
 */
export const sqlHistoryForCredentials = new PreparedQuery<ISqlHistoryForCredentialsParams,ISqlHistoryForCredentialsResult>(sqlHistoryForCredentialsIR);


