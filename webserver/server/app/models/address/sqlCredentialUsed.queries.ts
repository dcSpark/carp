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

const sqlCredentialUsedIR: any = {"name":"sqlCredentialUsed","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":143,"b":153,"line":5,"col":39}]}},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":198,"b":208,"line":7,"col":36}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":252,"b":262,"line":9,"col":35}]}}],"usedParamSet":{"credentials":true,"until_tx_id":true,"after_tx_id":true},"statement":{"body":"SELECT DISTINCT \"StakeCredential\".credential\nFROM \"StakeCredential\"\nWHERE\n  \"StakeCredential\".credential = ANY (:credentials)\n  AND\n  (\"StakeCredential\".first_tx) <= (:until_tx_id)\n  AND\n  (\"StakeCredential\".first_tx) > (:after_tx_id)","loc":{"a":30,"b":263,"line":2,"col":0}}};

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


