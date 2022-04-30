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

const sqlHistoryForAddressesIR: any = {"name":"sqlHistoryForAddresses","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":604,"b":612,"line":17,"col":34}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":698,"b":711,"line":20,"col":24}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":863,"b":876,"line":26,"col":25},{"a":1016,"b":1029,"line":29,"col":26}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1057,"b":1067,"line":29,"col":67}]}},{"name":"limit","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1174,"b":1178,"line":34,"col":14}]}}],"usedParamSet":{"addresses":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true,"limit":true},"statement":{"body":"SELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM \"Address\"\n      INNER JOIN \"TransactionOutput\" ON \"TransactionOutput\".address_id = \"Address\".id\n      INNER JOIN \"Transaction\" ON \"TransactionOutput\".tx_id = \"Transaction\".id\n      INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n      WHERE\n        \"Address\".payload = ANY (:addresses)\n        AND\n                                              \n        \"Block\".id <= (:until_block_id)\n        and (\n                                                                                                             \n          \"Block\".id > (:after_block_id)\n            or\n                                                                                               \n          (\"Block\".id = (:after_block_id) and \"Transaction\".id > (:after_tx_id))\n        ) \n      ORDER BY\n        \"Block\".height ASC,\n        \"Transaction\".tx_index ASC\n      LIMIT (:limit)","loc":{"a":35,"b":1179,"line":2,"col":0}}};

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
 *       FROM "Address"
 *       INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
 *       INNER JOIN "Transaction" ON "TransactionOutput".tx_id = "Transaction".id
 *       INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *       WHERE
 *         "Address".payload = ANY (:addresses)
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
export const sqlHistoryForAddresses = new PreparedQuery<ISqlHistoryForAddressesParams,ISqlHistoryForAddressesResult>(sqlHistoryForAddressesIR);


