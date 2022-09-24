/** Types generated for queries found in "app/models/address/sqlCredentialUsed.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlCredentialUsed' parameters type */
export interface ISqlCredentialUsedParams {
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  until_tx_id: string | null | void;
}

/** 'SqlCredentialUsed' return type */
export interface ISqlCredentialUsedResult {
  credential: Buffer;
}

/** 'SqlCredentialUsed' query type */
export interface ISqlCredentialUsedQuery {
  params: ISqlCredentialUsedParams;
  result: ISqlCredentialUsedResult;
}

const sqlCredentialUsedIR: any = {"usedParamSet":{"credentials":true,"until_tx_id":true,"after_tx_id":true},"params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"locs":[{"a":112,"b":123}]},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":167,"b":178}]},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":221,"b":232}]}],"statement":"SELECT DISTINCT \"StakeCredential\".credential\nFROM \"StakeCredential\"\nWHERE\n  \"StakeCredential\".credential = ANY (:credentials)\n  AND\n  (\"StakeCredential\".first_tx) <= (:until_tx_id)\n  AND\n  (\"StakeCredential\".first_tx) > (:after_tx_id)"};

/**
 * Query generated from SQL:
 * ```
 * SELECT DISTINCT "StakeCredential".credential
 * FROM "StakeCredential"
 * WHERE
 *   "StakeCredential".credential = ANY (:credentials)
 *   AND
 *   ("StakeCredential".first_tx) <= (:until_tx_id)
 *   AND
 *   ("StakeCredential".first_tx) > (:after_tx_id)
 * ```
 */
export const sqlCredentialUsed = new PreparedQuery<ISqlCredentialUsedParams,ISqlCredentialUsedResult>(sqlCredentialUsedIR);


