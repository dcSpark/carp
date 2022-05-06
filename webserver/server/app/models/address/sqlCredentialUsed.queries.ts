/** Types generated for queries found in "app/models/address/sqlCredentialUsed.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlCredentialUsed' parameters type */
export interface ISqlCredentialUsedParams {
  after_block_id: number | null | void;
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  relation: number | null | void;
  until_block_id: number | null | void;
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

const sqlCredentialUsedIR: any = {"name":"sqlCredentialUsed","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":374,"b":384,"line":8,"col":39}]}},{"name":"relation","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":432,"b":439,"line":10,"col":39}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":524,"b":537,"line":13,"col":30}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":667,"b":680,"line":19,"col":31},{"a":814,"b":827,"line":22,"col":32}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":855,"b":865,"line":22,"col":73}]}}],"usedParamSet":{"credentials":true,"relation":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true},"statement":{"body":"SELECT DISTINCT \"StakeCredential\".credential\nFROM \"StakeCredential\"\nINNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\nINNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\nINNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n  \"StakeCredential\".credential = ANY (:credentials)\n  AND\n  (\"TxCredentialRelation\".relation & (:relation)) > 0\n  AND\n                                        \n  \"Transaction\".block_id <= (:until_block_id)\n  AND (\n                                                                                       \n    \"Transaction\".block_id > (:after_block_id)\n      OR\n                                                                                         \n    (\"Transaction\".block_id = (:after_block_id) AND \"Transaction\".id > (:after_tx_id))\n  )","loc":{"a":30,"b":871,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT DISTINCT "StakeCredential".credential
 * FROM "StakeCredential"
 * INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 * INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 * INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *   "StakeCredential".credential = ANY (:credentials)
 *   AND
 *   ("TxCredentialRelation".relation & (:relation)) > 0
 *   AND
 *                                         
 *   "Transaction".block_id <= (:until_block_id)
 *   AND (
 *                                                                                        
 *     "Transaction".block_id > (:after_block_id)
 *       OR
 *                                                                                          
 *     ("Transaction".block_id = (:after_block_id) AND "Transaction".id > (:after_tx_id))
 *   )
 * ```
 */
export const sqlCredentialUsed = new PreparedQuery<ISqlCredentialUsedParams,ISqlCredentialUsedResult>(sqlCredentialUsedIR);


