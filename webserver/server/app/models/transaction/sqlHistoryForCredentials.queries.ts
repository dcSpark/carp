/** Types generated for queries found in "app/models/transaction/sqlHistoryForCredentials.sql" */
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

const sqlHistoryForCredentialsIR: any = {"name":"sqlHistoryForCredentials","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":458,"b":468,"line":10,"col":43}]}},{"name":"relation","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":524,"b":531,"line":12,"col":43}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":628,"b":641,"line":15,"col":34}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":795,"b":808,"line":21,"col":35},{"a":954,"b":967,"line":24,"col":36}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":995,"b":1005,"line":24,"col":77}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1087,"b":1091,"line":30,"col":12}]}}],"usedParamSet":{"credentials":true,"relation":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"WITH related_txs AS (\n  SELECT * FROM (\n    SELECT DISTINCT ON (\"Transaction\".id) \"Transaction\".*\n    FROM \"StakeCredential\"\n    INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n    INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n    INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n    WHERE\n      \"StakeCredential\".credential = ANY (:credentials)\n      AND\n      (\"TxCredentialRelation\".relation & (:relation)) > 0\n      AND\n                                            \n      \"Transaction\".block_id <= (:until_block_id)\n      AND (\n                                                                                                       \n        \"Transaction\".block_id > (:after_block_id)\n          OR\n                                                                                             \n        (\"Transaction\".block_id = (:after_block_id) AND \"Transaction\".id > (:after_tx_id))\n      )\n  ) t\n    ORDER BY\n      block_id ASC,\n      tx_index ASC\n    LIMIT (:limit)\n)\nSELECT related_txs.id,\n        related_txs.payload,\n        related_txs.hash,\n        related_txs.tx_index,\n        related_txs.is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM related_txs\n      INNER JOIN \"Block\" ON related_txs.block_id = \"Block\".id","loc":{"a":37,"b":1442,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH related_txs AS (
 *   SELECT * FROM (
 *     SELECT DISTINCT ON ("Transaction".id) "Transaction".*
 *     FROM "StakeCredential"
 *     INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *     INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *     INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *     WHERE
 *       "StakeCredential".credential = ANY (:credentials)
 *       AND
 *       ("TxCredentialRelation".relation & (:relation)) > 0
 *       AND
 *                                             
 *       "Transaction".block_id <= (:until_block_id)
 *       AND (
 *                                                                                                        
 *         "Transaction".block_id > (:after_block_id)
 *           OR
 *                                                                                              
 *         ("Transaction".block_id = (:after_block_id) AND "Transaction".id > (:after_tx_id))
 *       )
 *   ) t
 *     ORDER BY
 *       block_id ASC,
 *       tx_index ASC
 *     LIMIT (:limit)
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
 * ```
 */
export const sqlHistoryForCredentials = new PreparedQuery<ISqlHistoryForCredentialsParams,ISqlHistoryForCredentialsResult>(sqlHistoryForCredentialsIR);


