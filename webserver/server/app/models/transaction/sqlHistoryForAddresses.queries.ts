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

const sqlHistoryForAddressesIR: any = {"name":"sqlHistoryForAddresses","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":127,"b":135,"line":6,"col":36}]}},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":409,"b":419,"line":13,"col":41},{"a":824,"b":834,"line":24,"col":40},{"a":1286,"b":1296,"line":35,"col":49}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":476,"b":486,"line":15,"col":40},{"a":890,"b":900,"line":26,"col":39},{"a":1361,"b":1371,"line":37,"col":48}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":552,"b":556,"line":17,"col":16},{"a":965,"b":969,"line":28,"col":16},{"a":1445,"b":1449,"line":39,"col":16},{"a":1960,"b":1964,"line":55,"col":8}]}}],"usedParamSet":{"addresses":true,"until_tx_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"WITH\n  address_row AS (\n    SELECT *\n    FROM \"Address\"\n    WHERE \"Address\".payload = ANY (:addresses)\n  ),\n  outputs AS (\n        SELECT DISTINCT ON (\"TransactionOutput\".tx_id) \"TransactionOutput\".tx_id\n        FROM \"TransactionOutput\"\n        INNER JOIN address_row ON \"TransactionOutput\".address_id = address_row.id\n        WHERE\n          \"TransactionOutput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionOutput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionOutput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  inputs AS (\n        SELECT DISTINCT ON (\"TransactionInput\".tx_id) \"TransactionInput\".tx_id\n        FROM \"TransactionInput\"\n        INNER JOIN address_row ON \"TransactionInput\".address_id = address_row.id\n        WHERE\n          \"TransactionInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionInput\".tx_id ASC\n        LIMIT (:limit)\n  ),\n  ref_inputs AS (\n        SELECT DISTINCT ON (\"TransactionReferenceInput\".tx_id) \"TransactionReferenceInput\".tx_id\n        FROM \"TransactionReferenceInput\"\n        INNER JOIN address_row ON \"TransactionReferenceInput\".address_id = address_row.id\n        WHERE\n          \"TransactionReferenceInput\".tx_id <= (:until_tx_id)\n          AND\n          \"TransactionReferenceInput\".tx_id > (:after_tx_id)\n        ORDER BY \"TransactionReferenceInput\".tx_id ASC\n        LIMIT (:limit)\n  )\nSELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\nFROM \"Transaction\"\nINNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \"Transaction\".id IN (SELECT * FROM inputs UNION ALL SELECT * from ref_inputs UNION ALL SELECT * from outputs)\nORDER BY \"Transaction\".id ASC\nLIMIT (:limit)","loc":{"a":35,"b":1965,"line":2,"col":0}}};

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


