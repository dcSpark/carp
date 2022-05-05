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

const sqlHistoryForCredentialsIR: any = {"name":"sqlHistoryForCredentials","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":426,"b":436,"line":9,"col":41}]}},{"name":"relation","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":488,"b":495,"line":11,"col":41}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":936,"b":949,"line":27,"col":24}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1101,"b":1114,"line":33,"col":25},{"a":1254,"b":1267,"line":36,"col":26}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1293,"b":1303,"line":36,"col":65}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1408,"b":1412,"line":41,"col":14}]}}],"usedParamSet":{"credentials":true,"relation":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"WITH related_txs AS (\n  SELECT DISTINCT ON (\"Transaction\".id) \"Transaction\".*\n  FROM \"StakeCredential\"\n  INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n  INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n  INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n  WHERE\n    \"StakeCredential\".credential = ANY (:credentials)\n    AND\n    (\"TxCredentialRelation\".relation & (:relation)) > 0\n)\nSELECT related_txs.id,\n        related_txs.payload,\n        related_txs.hash,\n        related_txs.tx_index,\n        related_txs.is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM related_txs\n      INNER JOIN \"Block\" ON related_txs.block_id = \"Block\".id\n      WHERE\n                                              \n        \"Block\".id <= (:until_block_id)\n        and (\n                                                                                                             \n          \"Block\".id > (:after_block_id)\n            or\n                                                                                               \n          (\"Block\".id = (:after_block_id) and related_txs.id > (:after_tx_id))\n        ) \n      ORDER BY\n        \"Block\".height ASC,\n        related_txs.tx_index ASC\n      LIMIT (:limit)","loc":{"a":37,"b":1413,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH related_txs AS (
 *   SELECT DISTINCT ON ("Transaction".id) "Transaction".*
 *   FROM "StakeCredential"
 *   INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *   INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *   INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *   WHERE
 *     "StakeCredential".credential = ANY (:credentials)
 *     AND
 *     ("TxCredentialRelation".relation & (:relation)) > 0
 * )
 * SELECT related_txs.id,
 *         related_txs.payload,
 *         related_txs.hash,
 *         related_txs.tx_index,
 *         related_txs.is_valid,
 *         "Block".hash AS block_hash,
 *         "Block".epoch,
 *         "Block".slot,
 *         "Block".era,
 *         "Block".height
 *       FROM related_txs
 *       INNER JOIN "Block" ON related_txs.block_id = "Block".id
 *       WHERE
 *                                               
 *         "Block".id <= (:until_block_id)
 *         and (
 *                                                                                                              
 *           "Block".id > (:after_block_id)
 *             or
 *                                                                                                
 *           ("Block".id = (:after_block_id) and related_txs.id > (:after_tx_id))
 *         ) 
 *       ORDER BY
 *         "Block".height ASC,
 *         related_txs.tx_index ASC
 *       LIMIT (:limit)
 * ```
 */
export const sqlHistoryForCredentials = new PreparedQuery<ISqlHistoryForCredentialsParams,ISqlHistoryForCredentialsResult>(sqlHistoryForCredentialsIR);


