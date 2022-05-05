/** Types generated for queries found in "app/models/sqlHistoryForCredentials.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlHistoryForCredentials' parameters type */
export interface ISqlHistoryForCredentialsParams {
  after_block_id: number | null | void;
  after_tx_id: string | null | void;
  credentials: BufferArray | null | void;
  limit: string | null | void;
  relation: number | null | void;
  until_block_id: number | null | void;
}

/** 'SqlHistoryForCredentials' return type */
export interface ISqlHistoryForCredentialsResult {
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

/** 'SqlHistoryForCredentials' query type */
export interface ISqlHistoryForCredentialsQuery {
  params: ISqlHistoryForCredentialsParams;
  result: ISqlHistoryForCredentialsResult;
}

const sqlHistoryForCredentialsIR: any = {"name":"sqlHistoryForCredentials","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":645,"b":655,"line":17,"col":45}]}},{"name":"relation","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":715,"b":722,"line":19,"col":45}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":813,"b":826,"line":22,"col":24}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":978,"b":991,"line":28,"col":25},{"a":1131,"b":1144,"line":31,"col":26}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1172,"b":1182,"line":31,"col":67}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1289,"b":1293,"line":36,"col":14}]}}],"usedParamSet":{"credentials":true,"relation":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"SELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM \"StakeCredential\"\n      INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n      INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n      INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n      WHERE\n        \"StakeCredential\".credential = ANY (:credentials)\n        AND\n        (\"TxCredentialRelation\".relation & (:relation)) > 0\n        AND\n                                              \n        \"Block\".id <= (:until_block_id)\n        and (\n                                                                                                             \n          \"Block\".id > (:after_block_id)\n            or\n                                                                                               \n          (\"Block\".id = (:after_block_id) and \"Transaction\".id > (:after_tx_id))\n        ) \n      ORDER BY\n        \"Block\".height ASC,\n        \"Transaction\".tx_index ASC\n      LIMIT (:limit)","loc":{"a":37,"b":1294,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
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
 *       FROM "StakeCredential"
 *       INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *       INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *       INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *       WHERE
 *         "StakeCredential".credential = ANY (:credentials)
 *         AND
 *         ("TxCredentialRelation".relation & (:relation)) > 0
 *         AND
 *                                               
 *         "Block".id <= (:until_block_id)
 *         and (
 *                                                                                                              
 *           "Block".id > (:after_block_id)
 *             or
 *                                                                                                
 *           ("Block".id = (:after_block_id) and "Transaction".id > (:after_tx_id))
 *         ) 
 *       ORDER BY
 *         "Block".height ASC,
 *         "Transaction".tx_index ASC
 *       LIMIT (:limit)
 * ```
 */
export const sqlHistoryForCredentials = new PreparedQuery<ISqlHistoryForCredentialsParams,ISqlHistoryForCredentialsResult>(sqlHistoryForCredentialsIR);


