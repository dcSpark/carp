/** Types generated for queries found in "app/models/address/sqlAddressUsed.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlAddressUsed' parameters type */
export interface ISqlAddressUsedParams {
  addresses: BufferArray | null | void;
  after_block_id: number | null | void;
  after_tx_id: string | null | void;
  until_block_id: number | null | void;
}

/** 'SqlAddressUsed' return type */
export interface ISqlAddressUsedResult {
  payload: Buffer;
}

/** 'SqlAddressUsed' query type */
export interface ISqlAddressUsedQuery {
  params: ISqlAddressUsedParams;
  result: ISqlAddressUsedResult;
}

const sqlAddressUsedIR: any = {"name":"sqlAddressUsed","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":396,"b":404,"line":8,"col":28}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":484,"b":497,"line":11,"col":30}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":627,"b":640,"line":17,"col":31},{"a":774,"b":787,"line":20,"col":32}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":815,"b":825,"line":20,"col":73}]}}],"usedParamSet":{"addresses":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true},"statement":{"body":"SELECT DISTINCT \"Address\".payload\nFROM \"Address\"\nINNER JOIN \"TransactionOutput\" ON \"TransactionOutput\".address_id = \"Address\".id\nLEFT JOIN \"TransactionInput\" ON \"TransactionInput\".utxo_id = \"TransactionOutput\".id\nINNER JOIN \"Transaction\" ON (\"TransactionOutput\".tx_id = \"Transaction\".id OR \"TransactionInput\".tx_id = \"Transaction\".id)\nWHERE\n  \"Address\".payload = ANY (:addresses)\n  AND\n                                        \n  \"Transaction\".block_id <= (:until_block_id)\n  AND (\n                                                                                       \n    \"Transaction\".block_id > (:after_block_id)\n      OR\n                                                                                         \n    (\"Transaction\".block_id = (:after_block_id) AND \"Transaction\".id > (:after_tx_id))\n  )","loc":{"a":27,"b":831,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT DISTINCT "Address".payload
 * FROM "Address"
 * INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
 * LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
 * INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
 * WHERE
 *   "Address".payload = ANY (:addresses)
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
export const sqlAddressUsed = new PreparedQuery<ISqlAddressUsedParams,ISqlAddressUsedResult>(sqlAddressUsedIR);


