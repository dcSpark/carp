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

const sqlAddressUsedIR: any = {"name":"sqlAddressUsed","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":473,"b":481,"line":8,"col":34}]}},{"name":"until_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":706,"b":719,"line":15,"col":37}]}},{"name":"after_block_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":856,"b":869,"line":21,"col":38},{"a":1010,"b":1023,"line":24,"col":39}]}},{"name":"after_tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":1061,"b":1071,"line":24,"col":90}]}}],"usedParamSet":{"addresses":true,"until_block_id":true,"after_block_id":true,"after_tx_id":true},"statement":{"body":"WITH address_tx_relations AS (\n  SELECT \"Address\".*, \"Transaction\".id as tx_id, \"Transaction\".block_id\n  FROM \"Address\"\n  INNER JOIN \"TransactionOutput\" ON \"TransactionOutput\".address_id = \"Address\".id\n  LEFT JOIN \"TransactionInput\" ON \"TransactionInput\".utxo_id = \"TransactionOutput\".id\n  INNER JOIN \"Transaction\" ON (\"TransactionOutput\".tx_id = \"Transaction\".id OR \"TransactionInput\".tx_id = \"Transaction\".id)\n  WHERE \"Address\".payload = ANY (:addresses)\n)\nSELECT DISTINCT address_tx_relations.payload\nFROM address_tx_relations\nINNER JOIN \"Block\" ON address_tx_relations.block_id = \"Block\".id\nWHERE\n                                        \n  address_tx_relations.block_id <= (:until_block_id)\n  and (\n                                                                                       \n    address_tx_relations.block_id > (:after_block_id)\n      or\n                                                                                         \n    (address_tx_relations.block_id = (:after_block_id) and address_tx_relations.tx_id > (:after_tx_id))\n  )","loc":{"a":27,"b":1077,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH address_tx_relations AS (
 *   SELECT "Address".*, "Transaction".id as tx_id, "Transaction".block_id
 *   FROM "Address"
 *   INNER JOIN "TransactionOutput" ON "TransactionOutput".address_id = "Address".id
 *   LEFT JOIN "TransactionInput" ON "TransactionInput".utxo_id = "TransactionOutput".id
 *   INNER JOIN "Transaction" ON ("TransactionOutput".tx_id = "Transaction".id OR "TransactionInput".tx_id = "Transaction".id)
 *   WHERE "Address".payload = ANY (:addresses)
 * )
 * SELECT DISTINCT address_tx_relations.payload
 * FROM address_tx_relations
 * INNER JOIN "Block" ON address_tx_relations.block_id = "Block".id
 * WHERE
 *                                         
 *   address_tx_relations.block_id <= (:until_block_id)
 *   and (
 *                                                                                        
 *     address_tx_relations.block_id > (:after_block_id)
 *       or
 *                                                                                          
 *     (address_tx_relations.block_id = (:after_block_id) and address_tx_relations.tx_id > (:after_tx_id))
 *   )
 * ```
 */
export const sqlAddressUsed = new PreparedQuery<ISqlAddressUsedParams,ISqlAddressUsedResult>(sqlAddressUsedIR);


