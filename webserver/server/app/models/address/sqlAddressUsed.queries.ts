/** Types generated for queries found in "app/models/address/sqlAddressUsed.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlAddressUsed' parameters type */
export interface ISqlAddressUsedParams {
  addresses: BufferArray | null | void;
  after_tx_id: string | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlAddressUsed' return type */
export interface ISqlAddressUsedResult {
  payload: Buffer | null;
}

/** 'SqlAddressUsed' query type */
export interface ISqlAddressUsedQuery {
  params: ISqlAddressUsedParams;
  result: ISqlAddressUsedResult;
}

const sqlAddressUsedIR: any = {"name":"sqlAddressUsed","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":119,"b":127,"line":6,"col":36}]}},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":344,"b":354,"line":13,"col":37},{"a":750,"b":760,"line":25,"col":39},{"a":944,"b":954,"line":29,"col":36}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":403,"b":413,"line":15,"col":36},{"a":1002,"b":1012,"line":31,"col":35}]}}],"usedParamSet":{"addresses":true,"until_tx_id":true,"after_tx_id":true},"statement":{"body":"WITH\n  address_row AS (\n    SELECT *\n    FROM \"Address\"\n    WHERE \"Address\".payload = ANY (:addresses)\n  ),\n  outputs AS (\n    SELECT DISTINCT address_row.payload\n    FROM \"TransactionOutput\"\n    INNER JOIN address_row ON \"TransactionOutput\".address_id = address_row.id\n    WHERE\n      \"TransactionOutput\".tx_id <= (:until_tx_id)\n      AND\n      \"TransactionOutput\".tx_id > (:after_tx_id)\n  ),\n  inputs AS (\n    SELECT DISTINCT address_row.payload\n    FROM \"TransactionInput\"\n    INNER JOIN (\n      SELECT \"TransactionOutput\".id, \"TransactionOutput\".address_id\n      FROM \"TransactionOutput\"\n      INNER JOIN address_row ON \"TransactionOutput\".address_id = address_row.id\n      WHERE\n        \"TransactionOutput\".tx_id <= (:until_tx_id)\n    ) spent_utxos ON \"TransactionInput\".utxo_id = spent_utxos.id\n    INNER JOIN address_row ON spent_utxos.address_id = address_row.id\n    WHERE\n      \"TransactionInput\".tx_id <= (:until_tx_id)\n      AND\n      \"TransactionInput\".tx_id > (:after_tx_id)\n  )\nSELECT DISTINCT all_address.payload\nFROM (SELECT * FROM inputs UNION ALL SELECT * from outputs) all_address","loc":{"a":27,"b":1125,"line":2,"col":0}}};

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
 *     SELECT DISTINCT address_row.payload
 *     FROM "TransactionOutput"
 *     INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
 *     WHERE
 *       "TransactionOutput".tx_id <= (:until_tx_id)
 *       AND
 *       "TransactionOutput".tx_id > (:after_tx_id)
 *   ),
 *   inputs AS (
 *     SELECT DISTINCT address_row.payload
 *     FROM "TransactionInput"
 *     INNER JOIN (
 *       SELECT "TransactionOutput".id, "TransactionOutput".address_id
 *       FROM "TransactionOutput"
 *       INNER JOIN address_row ON "TransactionOutput".address_id = address_row.id
 *       WHERE
 *         "TransactionOutput".tx_id <= (:until_tx_id)
 *     ) spent_utxos ON "TransactionInput".utxo_id = spent_utxos.id
 *     INNER JOIN address_row ON spent_utxos.address_id = address_row.id
 *     WHERE
 *       "TransactionInput".tx_id <= (:until_tx_id)
 *       AND
 *       "TransactionInput".tx_id > (:after_tx_id)
 *   )
 * SELECT DISTINCT all_address.payload
 * FROM (SELECT * FROM inputs UNION ALL SELECT * from outputs) all_address
 * ```
 */
export const sqlAddressUsed = new PreparedQuery<ISqlAddressUsedParams,ISqlAddressUsedResult>(sqlAddressUsedIR);


