/** Types generated for queries found in "app/models/address/sqlCredentialUsed.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlCredentialUsed' parameters type */
export interface ISqlCredentialUsedParams {
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  relation: number | null | void;
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

const sqlCredentialUsedIR: any = {"name":"sqlCredentialUsed","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":240,"b":250,"line":6,"col":39}]}},{"name":"relation","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":298,"b":305,"line":8,"col":39}]}},{"name":"until_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":357,"b":367,"line":10,"col":38}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":413,"b":423,"line":12,"col":37}]}}],"usedParamSet":{"credentials":true,"relation":true,"until_tx_id":true,"after_tx_id":true},"statement":{"body":"SELECT DISTINCT \"StakeCredential\".credential\nFROM \"StakeCredential\"\nINNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\nWHERE\n  \"StakeCredential\".credential = ANY (:credentials)\n  AND\n  (\"TxCredentialRelation\".relation & (:relation)) > 0\n  AND\n  (\"TxCredentialRelation\".tx_id) <= (:until_tx_id)\n  AND\n  (\"TxCredentialRelation\".tx_id) > (:after_tx_id)","loc":{"a":30,"b":424,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT DISTINCT "StakeCredential".credential
 * FROM "StakeCredential"
 * INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 * WHERE
 *   "StakeCredential".credential = ANY (:credentials)
 *   AND
 *   ("TxCredentialRelation".relation & (:relation)) > 0
 *   AND
 *   ("TxCredentialRelation".tx_id) <= (:until_tx_id)
 *   AND
 *   ("TxCredentialRelation".tx_id) > (:after_tx_id)
 * ```
 */
export const sqlCredentialUsed = new PreparedQuery<ISqlCredentialUsedParams,ISqlCredentialUsedResult>(sqlCredentialUsedIR);


