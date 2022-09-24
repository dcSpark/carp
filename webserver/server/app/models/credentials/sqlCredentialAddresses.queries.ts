/** Types generated for queries found in "app/models/credentials/sqlCredentialAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlCredentialAddresses' parameters type */
export interface ISqlCredentialAddressesParams {
  after_address: Buffer | null | void;
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

const sqlCredentialAddressesIR: any = {"usedParamSet":{"until_tx_id":true,"after_address":true,"credentials":true,"double_limit":true,"limit":true},"params":[{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":103,"b":114}]},{"name":"after_address","required":false,"transform":{"type":"scalar"},"locs":[{"a":233,"b":246},{"a":290,"b":303},{"a":436,"b":449}]},{"name":"credentials","required":false,"transform":{"type":"scalar"},"locs":[{"a":745,"b":756}]},{"name":"double_limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":1007,"b":1019}]},{"name":"limit","required":false,"transform":{"type":"scalar"},"locs":[{"a":1194,"b":1199}]}],"statement":"WITH\n  max_address_id AS (\n    SELECT \"Address\".id\n    FROM \"Address\"\n    WHERE \"Address\".first_tx <= (:until_tx_id)\n    ORDER BY \"Address\".first_tx DESC\n    LIMIT 1\n  ),\n  min_address_id AS (\n    SELECT\n      CASE\n            WHEN (:after_address)::bytea IS NULL then -1\n            WHEN (:after_address)::bytea IS NOT NULL then (\n              SELECT \"Address\".id\n              FROM \"Address\"\n              WHERE \"Address\".payload = (:after_address)::bytea\n            )\n      END\n  ),\n  relations AS (\n    SELECT \"AddressCredentialRelation\".address_id\n    FROM \"StakeCredential\"\n    INNER JOIN \"AddressCredentialRelation\" ON \"StakeCredential\".id = \"AddressCredentialRelation\".credential_id\n    WHERE\n      \"StakeCredential\".credential = ANY (:credentials)\n      AND\n      \"AddressCredentialRelation\".address_id > (SELECT * FROM min_address_id)\n      AND\n      \"AddressCredentialRelation\".address_id <= (SELECT * FROM max_address_id)\n      ORDER BY \"AddressCredentialRelation\".address_id ASC\n      LIMIT (:double_limit)\n  )\nSELECT DISTINCT ON (\"Address\".id) \"Address\".payload, \"Address\".first_tx\nFROM \"Address\"\nWHERE \"Address\".id in (SELECT * FROM relations)\nORDER BY \"Address\".id ASC\nLIMIT (:limit)"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *   max_address_id AS (
 *     SELECT "Address".id
 *     FROM "Address"
 *     WHERE "Address".first_tx <= (:until_tx_id)
 *     ORDER BY "Address".first_tx DESC
 *     LIMIT 1
 *   ),
 *   min_address_id AS (
 *     SELECT
 *       CASE
 *             WHEN (:after_address)::bytea IS NULL then -1
 *             WHEN (:after_address)::bytea IS NOT NULL then (
 *               SELECT "Address".id
 *               FROM "Address"
 *               WHERE "Address".payload = (:after_address)::bytea
 *             )
 *       END
 *   ),
 *   relations AS (
 *     SELECT "AddressCredentialRelation".address_id
 *     FROM "StakeCredential"
 *     INNER JOIN "AddressCredentialRelation" ON "StakeCredential".id = "AddressCredentialRelation".credential_id
 *     WHERE
 *       "StakeCredential".credential = ANY (:credentials)
 *       AND
 *       "AddressCredentialRelation".address_id > (SELECT * FROM min_address_id)
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


