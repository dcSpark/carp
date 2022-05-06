/** Types generated for queries found in "app/models/transaction/sqlHistoryForAddresses.sql" */
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

const sqlHistoryForAddressesIR: any = {"name":"sqlHistoryForAddresses","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":486,"b":494,"line":9,"col":36}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":586,"b":599,"line":12,"col":34}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":753,"b":766,"line":18,"col":35},{"a":912,"b":925,"line":21,"col":36}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":953,"b":963,"line":21,"col":77}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1037,"b":1041,"line":27,"col":10}]}}],"usedParamSet":{"addresses":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"WITH related_txs AS (\n  SELECT * FROM (\n    SELECT DISTINCT ON (\"Transaction\".id) \"Transaction\".*\n    FROM \"Address\"\n    INNER JOIN \"TransactionOutput\" ON \"TransactionOutput\".address_id = \"Address\".id\n    LEFT JOIN \"TransactionInput\" ON \"TransactionInput\".utxo_id = \"TransactionOutput\".id\n    INNER JOIN \"Transaction\" ON (\"TransactionOutput\".tx_id = \"Transaction\".id OR \"TransactionInput\".tx_id = \"Transaction\".id)\n    WHERE \"Address\".payload = ANY (:addresses)\n      AND\n                                            \n      \"Transaction\".block_id <= (:until_block_id)\n      AND (\n                                                                                                       \n        \"Transaction\".block_id > (:after_block_id)\n          OR\n                                                                                             \n        (\"Transaction\".block_id = (:after_block_id) AND \"Transaction\".id > (:after_tx_id))\n      )\n  ) t\n  ORDER BY\n    block_id ASC,\n    tx_index ASC\n  LIMIT (:limit)\n)\nSELECT related_txs.id,\n        related_txs.payload,\n        related_txs.hash,\n        related_txs.tx_index,\n        related_txs.is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM related_txs\n      INNER JOIN \"Block\" ON related_txs.block_id = \"Block\".id","loc":{"a":35,"b":1392,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH related_txs AS (
 *   SELECT * FROM (
 *     SELECT DISTINCT ON ("Transaction".id) "Transaction".*
 *     FROM "Address"
 *     INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
 *     LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
 *     INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
 *     WHERE "Address".payload = ANY (:addresses)
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
 *   ORDER BY
 *     block_id ASC,
 *     tx_index ASC
 *   LIMIT (:limit)
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
export const sqlHistoryForAddresses = new PreparedQuery<ISqlHistoryForAddressesParams,ISqlHistoryForAddressesResult>(sqlHistoryForAddressesIR);


