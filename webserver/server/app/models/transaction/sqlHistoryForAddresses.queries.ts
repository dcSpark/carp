/** Types generated for queries found in "app/models/transaction/sqlHistoryForAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlHistoryForAddresses' parameters type */
export interface ISqlHistoryForAddressesParams {
  addresses: BufferArray | null | void;
  after_tx_id: string | null | void;
  limit: string | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlHistoryForAddresses' return type */
export interface ISqlHistoryForAddressesResult {
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

/** 'SqlHistoryForAddresses' query type */
export interface ISqlHistoryForAddressesQuery {
  params: ISqlHistoryForAddressesParams;
  result: ISqlHistoryForAddressesResult;
}

const sqlHistoryForAddressesIR: any = {"usedParamSet":{"addresses":true,"until_tx_id":true,"after_tx_id":true,"limit":true},"params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"locs":[{"a":91,"b":100}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":373,"b":384},{"a":788,"b":799},{"a":1250,"b":1261}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":440,"b":451},{"a":854,"b":865},{"a":1325,"b":1336}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":516,"b":521},{"a":929,"b":934},{"a":1409,"b":1414},{"a":1924,"b":1929}]}],"statement":"WITH\n  address_row AS (\n    SELECT *\n    FROM \"Address\"\n    WHERE \"Address\".payload = ANY (:addresses)\n  ),\n  outputs AS (\n        SELECT DISTINCT ON (\"TransactionOutput\".tx_id) \"TransactionOutput\".tx_id\n        FROM \"TransactionOutput\"\n        INNER JOIN address_row ON \"TransactionOutput\".address_id = address_row.id\n        WHERE\n          \"TransactionOutput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionOutput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionOutput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  inputs AS (\n        SELECT DISTINCT ON (\"TransactionInput\".tx_id) \"TransactionInput\".tx_id\n        FROM \"TransactionInput\"\n        INNER JOIN address_row ON \"TransactionInput\".address_id = address_row.id\n        WHERE\n          \"TransactionInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionInput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  ref_inputs AS (\n        SELECT DISTINCT ON (\"TransactionReferenceInput\".tx_id) \"TransactionReferenceInput\".tx_id\n        FROM \"TransactionReferenceInput\"\n        INNER JOIN address_row ON \"TransactionReferenceInput\".address_id = address_row.id\n        WHERE\n          \"TransactionReferenceInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionReferenceInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionReferenceInput\".tx_id ASC\n        LIMIT (:limit)\n  )\nSELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\nFROM \"Transaction\"\nINNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \"Transaction\".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)\nORDER BY \"Transaction\".id ASC\nLIMIT (:limit)"};

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
 * FROM "Transaction"
 * INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE "Transaction".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)
 * ORDER BY "Transaction".id ASC
 * LIMIT (:limit)
 * ```
 */
export const sqlHistoryForAddresses = new PreparedQuery<ISqlHistoryForAddressesParams,ISqlHistoryForAddressesResult>(sqlHistoryForAddressesIR);


