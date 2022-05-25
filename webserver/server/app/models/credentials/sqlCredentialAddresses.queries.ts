/** Types generated for queries found in "app/models/credentials/sqlCredentialAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlCredentialAddresses' parameters type */
export interface ISqlCredentialAddressesParams {
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  double_limit: string | null | void;
  limit: string | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlCredentialAddresses' return type */
export interface ISqlCredentialAddressesResult {
  first_tx: string;
  payload: Buffer;
}

/** 'SqlCredentialAddresses' query type */
export interface ISqlCredentialAddressesQuery {
  params: ISqlCredentialAddressesParams;
  result: ISqlCredentialAddressesResult;
}

const sqlCredentialAddressesIR: any = {"name":"sqlCredentialAddresses","params":[{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":144,"b":154,"line":6,"col":34}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":265,"b":275,"line":11,"col":33}]}},{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":541,"b":551,"line":18,"col":43}]}},{"name":"double_limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":804,"b":815,"line":24,"col":14}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":991,"b":995,"line":30,"col":8}]}}],"usedParamSet":{"until_tx_id":true,"after_tx_id":true,"credentials":true,"double_limit":true,"limit":true},"statement":{"body":"WITH\n  max_address_id AS (\n    SELECT MAX(\"Address\".id)\n    FROM \"Address\"\n    WHERE \"Address\".first_tx <= (:until_tx_id)\n  ),\n  min_address_id AS (\n    SELECT MIN(\"Address\".id)\n    FROM \"Address\"\n    WHERE \"Address\".first_tx > (:after_tx_id)\n  ),\n  relations AS (\n    SELECT \"AddressCredentialRelation\".address_id\n    FROM \"StakeCredential\"\n    INNER JOIN \"AddressCredentialRelation\" ON \"StakeCredential\".id = \"AddressCredentialRelation\".credential_id\n    WHERE\n      \"StakeCredential\".credential = ANY (:credentials)\n      AND\n      \"AddressCredentialRelation\".address_id >= (SELECT * FROM min_address_id)\n      AND\n      \"AddressCredentialRelation\".address_id <= (SELECT * FROM max_address_id)\n      ORDER BY \"AddressCredentialRelation\".address_id ASC\n      LIMIT (:double_limit)\n  )\nSELECT DISTINCT ON (\"Address\".id) \"Address\".payload, \"Address\".first_tx\nFROM \"Address\"\nWHERE \"Address\".id in (SELECT * FROM relations)\nORDER BY \"Address\".id ASC\nLIMIT (:limit)","loc":{"a":35,"b":996,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   max_address_id AS (
 *     SELECT MAX("Address".id)
 *     FROM "Address"
 *     WHERE "Address".first_tx <= (:until_tx_id)
 *   ),
 *   min_address_id AS (
 *     SELECT MIN("Address".id)
 *     FROM "Address"
 *     WHERE "Address".first_tx > (:after_tx_id)
 *   ),
 *   relations AS (
 *     SELECT "AddressCredentialRelation".address_id
 *     FROM "StakeCredential"
 *     INNER JOIN "AddressCredentialRelation" ON "StakeCredential".id = "AddressCredentialRelation".credential_id
 *     WHERE
 *       "StakeCredential".credential = ANY (:credentials)
 *       AND
 *       "AddressCredentialRelation".address_id >= (SELECT * FROM min_address_id)
 *       AND
 *       "AddressCredentialRelation".address_id <= (SELECT * FROM max_address_id)
 *       ORDER BY "AddressCredentialRelation".address_id ASC
 *       LIMIT (:double_limit)
 *   )
 * SELECT DISTINCT ON ("Address".id) "Address".payload, "Address".first_tx
 * FROM "Address"
 * WHERE "Address".id in (SELECT * FROM relations)
 * ORDER BY "Address".id ASC
 * LIMIT (:limit)
 * ```
 */
export const sqlCredentialAddresses = new PreparedQuery<ISqlCredentialAddressesParams,ISqlCredentialAddressesResult>(sqlCredentialAddressesIR);


