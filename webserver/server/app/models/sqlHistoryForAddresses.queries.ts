/** Types generated for queries found in "app/models/sqlHistoryForAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlHistoryForAddresses' parameters type */
export interface ISqlHistoryForAddressesParams {
  addresses: BufferArray | null | void;
  after_block_id: number | null | void;
  after_tx_id: string | null | void;
  limit: string | null | void;
  until_block_id: number | null | void;
}

/** 'SqlHistoryForAddresses' return type */
export interface ISqlHistoryForAddressesResult {
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

/** 'SqlHistoryForAddresses' query type */
export interface ISqlHistoryForAddressesQuery {
  params: ISqlHistoryForAddressesParams;
  result: ISqlHistoryForAddressesResult;
}

const sqlHistoryForAddressesIR: any = {"name":"sqlHistoryForAddresses","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":456,"b":464,"line":8,"col":34}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":900,"b":913,"line":24,"col":24}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1065,"b":1078,"line":30,"col":25},{"a":1218,"b":1231,"line":33,"col":26}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1257,"b":1267,"line":33,"col":65}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1372,"b":1376,"line":38,"col":14}]}}],"usedParamSet":{"addresses":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"WITH related_txs AS (\n  SELECT DISTINCT ON (\"Transaction\".id) \"Transaction\".*\n  FROM \"Address\"\n  INNER JOIN \"TransactionOutput\" ON \"TransactionOutput\".address_id = \"Address\".id\n  LEFT JOIN \"TransactionInput\" ON \"TransactionInput\".utxo_id = \"TransactionOutput\".id\n  INNER JOIN \"Transaction\" ON (\"TransactionOutput\".tx_id = \"Transaction\".id OR \"TransactionInput\".tx_id = \"Transaction\".id)\n  WHERE \"Address\".payload = ANY (:addresses)\n)\nSELECT related_txs.id,\n        related_txs.payload,\n        related_txs.hash,\n        related_txs.tx_index,\n        related_txs.is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM related_txs\n      INNER JOIN \"Block\" ON related_txs.block_id = \"Block\".id\n      WHERE\n                                              \n        \"Block\".id <= (:until_block_id)\n        and (\n                                                                                                             \n          \"Block\".id > (:after_block_id)\n            or\n                                                                                               \n          (\"Block\".id = (:after_block_id) and related_txs.id > (:after_tx_id))\n        ) \n      ORDER BY\n        \"Block\".height ASC,\n        related_txs.tx_index ASC\n      LIMIT (:limit)","loc":{"a":35,"b":1377,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH related_txs AS (
 *   SELECT DISTINCT ON ("Transaction".id) "Transaction".*
 *   FROM "Address"
 *   INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
 *   LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
 *   INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
 *   WHERE "Address".payload = ANY (:addresses)
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
export const sqlHistoryForAddresses = new PreparedQuery<ISqlHistoryForAddressesParams,ISqlHistoryForAddressesResult>(sqlHistoryForAddressesIR);


