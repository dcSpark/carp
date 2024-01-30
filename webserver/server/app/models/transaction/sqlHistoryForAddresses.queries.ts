/** Types generated for queries found in "app/models/transaction/sqlHistoryForAddresses.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type BufferArray = (Buffer)[];

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'SqlHistoryForAddresses' parameters type */
export interface ISqlHistoryForAddressesParams {
  addresses?: BufferArray | null | void;
  after_tx_id?: NumberOrString | null | void;
  limit?: NumberOrString | null | void;
  until_tx_id?: NumberOrString | null | void;
  with_input_context: boolean;
}

/** 'SqlHistoryForAddresses' return type */
export interface ISqlHistoryForAddressesResult {
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

/** 'SqlHistoryForAddresses' query type */
export interface ISqlHistoryForAddressesQuery {
  params: ISqlHistoryForAddressesParams;
  result: ISqlHistoryForAddressesResult;
}

const sqlHistoryForAddressesIR: any = {"usedParamSet":{"addresses":true,"until_tx_id":true,"after_tx_id":true,"limit":true,"with_input_context":true},"params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"locs":[{"a":91,"b":100}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":373,"b":384},{"a":788,"b":799},{"a":1250,"b":1261}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":440,"b":451},{"a":854,"b":865},{"a":1325,"b":1336}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":516,"b":521},{"a":929,"b":934},{"a":1409,"b":1414},{"a":2215,"b":2220},{"a":3446,"b":3451}]},{"name":"with_input_context","required":true,"transform":{"type":"scalar"},"locs":[{"a":3493,"b":3512},{"a":3576,"b":3595}]}],"statement":"WITH\n  address_row AS (\n    SELECT *\n    FROM \"Address\"\n    WHERE \"Address\".payload = ANY (:addresses)\n  ),\n  outputs AS (\n        SELECT DISTINCT ON (\"TransactionOutput\".tx_id) \"TransactionOutput\".tx_id\n        FROM \"TransactionOutput\"\n        INNER JOIN address_row ON \"TransactionOutput\".address_id = address_row.id\n        WHERE\n          \"TransactionOutput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionOutput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionOutput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  inputs AS (\n        SELECT DISTINCT ON (\"TransactionInput\".tx_id) \"TransactionInput\".tx_id\n        FROM \"TransactionInput\"\n        INNER JOIN address_row ON \"TransactionInput\".address_id = address_row.id\n        WHERE\n          \"TransactionInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionInput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  ref_inputs AS (\n        SELECT DISTINCT ON (\"TransactionReferenceInput\".tx_id) \"TransactionReferenceInput\".tx_id\n        FROM \"TransactionReferenceInput\"\n        INNER JOIN address_row ON \"TransactionReferenceInput\".address_id = address_row.id\n        WHERE\n          \"TransactionReferenceInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionReferenceInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionReferenceInput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  base_query AS (\n        SELECT \"Transaction\".id,\n            \"Transaction\".payload as \"payload!\",\n            \"Transaction\".hash as \"hash!\",\n            \"Transaction\".tx_index as \"tx_index!\",\n            \"Transaction\".is_valid as \"is_valid!\",\n            \"Block\".hash AS \"block_hash!\",\n            \"Block\".epoch as \"epoch!\",\n            \"Block\".slot as \"slot!\",\n            \"Block\".era as \"era!\",\n            \"Block\".height as \"height!\",\n            NULL :: bytea as metadata,\n            NULL :: json as input_addresses\n        FROM \"Transaction\"\n        INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n        WHERE \"Transaction\".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)\n        ORDER BY \"Transaction\".id ASC\n        LIMIT (:limit)\n  ),\n  query_with_inputs_and_metadata AS (\n        SELECT \"Transaction\".id,\n                \"Transaction\".payload as \"payload!\",\n                \"Transaction\".hash as \"hash!\",\n                \"Transaction\".tx_index as \"tx_index!\",\n                \"Transaction\".is_valid as \"is_valid!\",\n                \"Block\".hash AS \"block_hash!\",\n                \"Block\".epoch as \"epoch!\",\n                \"Block\".slot as \"slot!\",\n                \"Block\".era as \"era!\",\n                \"Block\".height as \"height!\",\n                \"TransactionMetadata\".payload AS metadata,\n                json_agg(DISTINCT \"Address\".PAYLOAD) input_addresses\n        FROM \"Transaction\"\n        INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n        INNER JOIN \"TransactionInput\" ON \"TransactionInput\".tx_id = \"Transaction\".id\n        INNER JOIN \"Address\" ON \"Address\".id = \"TransactionInput\".address_id\n        LEFT JOIN \"TransactionMetadata\" ON \"Transaction\".id = \"TransactionMetadata\".tx_id\n        WHERE \"Transaction\".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)\n        GROUP BY \"Transaction\".id, \"Block\".id, \"TransactionMetadata\".id\n        ORDER BY \"Transaction\".id ASC\n        LIMIT (:limit)\n  )\nSELECT * FROM base_query WHERE NOT :with_input_context!\nUNION ALL\n(SELECT * from query_with_inputs_and_metadata WHERE :with_input_context!)"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   address_row AS (
 *     SELECT *
 *     FROM "Address"
 *     WHERE "Address".payload = ANY (:addresses)
 *   ),
 *   outputs AS (
 *         SELECT DISTINCT ON ("TransactionOutput".tx_id) "TransactionOutput".tx_id
 *         FROM "TransactionOutput"
 *         INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
 *         WHERE
 *           "TransactionOutput".tx_id <= (:until_tx_id)
 *           AND
 *           "TransactionOutput".tx_id > (:after_tx_id)
 *         ORDER BY "TransactionOutput".tx_id ASC
 *         LIMIT (:limit)
 *   ),
 *   inputs AS (
 *         SELECT DISTINCT ON ("TransactionInput".tx_id) "TransactionInput".tx_id
 *         FROM "TransactionInput"
 *         INNER JOIN address_row ON "TransactionInput".address_id = address_row.id
 *         WHERE
 *           "TransactionInput".tx_id <= (:until_tx_id)
 *           AND
 *           "TransactionInput".tx_id > (:after_tx_id)
 *         ORDER BY "TransactionInput".tx_id ASC
 *         LIMIT (:limit)
 *   ),
 *   ref_inputs AS (
 *         SELECT DISTINCT ON ("TransactionReferenceInput".tx_id) "TransactionReferenceInput".tx_id
 *         FROM "TransactionReferenceInput"
 *         INNER JOIN address_row ON "TransactionReferenceInput".address_id = address_row.id
 *         WHERE
 *           "TransactionReferenceInput".tx_id <= (:until_tx_id)
 *           AND
 *           "TransactionReferenceInput".tx_id > (:after_tx_id)
 *         ORDER BY "TransactionReferenceInput".tx_id ASC
 *         LIMIT (:limit)
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
 *         FROM "Transaction"
 *         INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *         WHERE "Transaction".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)
 *         ORDER BY "Transaction".id ASC
 *         LIMIT (:limit)
 *   ),
 *   query_with_inputs_and_metadata AS (
 *         SELECT "Transaction".id,
 *                 "Transaction".payload as "payload!",
 *                 "Transaction".hash as "hash!",
 *                 "Transaction".tx_index as "tx_index!",
 *                 "Transaction".is_valid as "is_valid!",
 *                 "Block".hash AS "block_hash!",
 *                 "Block".epoch as "epoch!",
 *                 "Block".slot as "slot!",
 *                 "Block".era as "era!",
 *                 "Block".height as "height!",
 *                 "TransactionMetadata".payload AS metadata,
 *                 json_agg(DISTINCT "Address".PAYLOAD) input_addresses
 *         FROM "Transaction"
 *         INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *         INNER JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
 *         INNER JOIN "Address" ON "Address".id = "TransactionInput".address_id
 *         LEFT JOIN "TransactionMetadata" ON "Transaction".id = "TransactionMetadata".tx_id
 *         WHERE "Transaction".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)
 *         GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
 *         ORDER BY "Transaction".id ASC
 *         LIMIT (:limit)
 *   )
 * SELECT * FROM base_query WHERE NOT :with_input_context!
 * UNION ALL
 * (SELECT * from query_with_inputs_and_metadata WHERE :with_input_context!)
 * ```
 */
export const sqlHistoryForAddresses = new PreparedQuery<ISqlHistoryForAddressesParams,ISqlHistoryForAddressesResult>(sqlHistoryForAddressesIR);


