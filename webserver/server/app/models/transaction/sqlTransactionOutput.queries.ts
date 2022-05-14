/** Types generated for queries found in "app/models/transaction/sqlTransactionOutput.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

export type numberArray = (number)[];

/** 'SqlTransactionOutput' parameters type */
export interface ISqlTransactionOutputParams {
  output_index: numberArray | null | void;
  tx_hash: BufferArray | null | void;
}

/** 'SqlTransactionOutput' return type */
export interface ISqlTransactionOutputResult {
  block_hash: Buffer;
  epoch: number;
  era: number;
  height: number;
  is_valid: boolean;
  slot: number;
  tx_index: number;
  utxo_payload: Buffer;
}

/** 'SqlTransactionOutput' query type */
export interface ISqlTransactionOutputQuery {
  params: ISqlTransactionOutputParams;
  result: ISqlTransactionOutputResult;
}

const sqlTransactionOutputIR: any = {"name":"sqlTransactionOutput","params":[{"name":"tx_hash","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":110,"b":116,"line":6,"col":8}]}},{"name":"output_index","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":137,"b":148,"line":7,"col":8}]}}],"usedParamSet":{"tx_hash":true,"output_index":true},"statement":{"body":"WITH pointers AS (\n  SELECT tx_hash, output_index\n  FROM\n    unnest(\n      (:tx_hash)::bytea[],\n      (:output_index)::int[]\n    ) x(tx_hash,output_index)\n)\nSELECT\n  \"TransactionOutput\".payload as utxo_payload,\n  \"Transaction\".is_valid,\n  \"Transaction\".tx_index,\n  \"Block\".hash AS block_hash,\n  \"Block\".epoch,\n  \"Block\".slot,\n  \"Block\".era,\n  \"Block\".height\nFROM\n  \"Transaction\"\n  INNER JOIN \"TransactionOutput\" ON \"Transaction\".id = \"TransactionOutput\".tx_id\n  INNER JOIN \"Block\" on \"Block\".id = \"Transaction\".block_id\nWHERE (\"Transaction\".hash, \"TransactionOutput\".output_index) in (SELECT tx_hash, output_index FROM pointers)","loc":{"a":33,"b":660,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * WITH pointers AS (
 *   SELECT tx_hash, output_index
 *   FROM
 *     unnest(
 *       (:tx_hash)::bytea[],
 *       (:output_index)::int[]
 *     ) x(tx_hash,output_index)
 * )
 * SELECT
 *   "TransactionOutput".payload as utxo_payload,
 *   "Transaction".is_valid,
 *   "Transaction".tx_index,
 *   "Block".hash AS block_hash,
 *   "Block".epoch,
 *   "Block".slot,
 *   "Block".era,
 *   "Block".height
 * FROM
 *   "Transaction"
 *   INNER JOIN "TransactionOutput" ON "Transaction".id = "TransactionOutput".tx_id
 *   INNER JOIN "Block" on "Block".id = "Transaction".block_id
 * WHERE ("Transaction".hash, "TransactionOutput".output_index) in (SELECT tx_hash, output_index FROM pointers)
 * ```
 */
export const sqlTransactionOutput = new PreparedQuery<ISqlTransactionOutputParams,ISqlTransactionOutputResult>(sqlTransactionOutputIR);


