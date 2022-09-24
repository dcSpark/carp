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
  payload: Buffer;
}

/** 'SqlAddressUsed' query type */
export interface ISqlAddressUsedQuery {
  params: ISqlAddressUsedParams;
  result: ISqlAddressUsedResult;
}

const sqlAddressUsedIR: any = {"usedParamSet":{"addresses":true,"until_tx_id":true,"after_tx_id":true},"params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"locs":[{"a":82,"b":91}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":127,"b":138}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":173,"b":184}]}],"statement":"SELECT DISTINCT \"Address\".payload\nFROM \"Address\"\nWHERE\n  \"Address\".payload = ANY (:addresses)\n  AND\n  (\"Address\".first_tx) <= (:until_tx_id)\n  AND\n  (\"Address\".first_tx) > (:after_tx_id)"};

/**
 * Query generated from SQL:
 * ```
 * SELECT DISTINCT "Address".payload
 * FROM "Address"
 * WHERE
 *   "Address".payload = ANY (:addresses)
 *   AND
 *   ("Address".first_tx) <= (:until_tx_id)
 *   AND
 *   ("Address".first_tx) > (:after_tx_id)
 * ```
 */
export const sqlAddressUsed = new PreparedQuery<ISqlAddressUsedParams,ISqlAddressUsedResult>(sqlAddressUsedIR);


