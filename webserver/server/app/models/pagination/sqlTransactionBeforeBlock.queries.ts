/** Types generated for queries found in "app/models/pagination/sqlTransactionBeforeBlock.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'SqlTransactionBeforeBlock' parameters type */
export interface ISqlTransactionBeforeBlockParams {
  until_block?: Buffer | null | void;
}

/** 'SqlTransactionBeforeBlock' return type */
export interface ISqlTransactionBeforeBlockResult {
  id: string;
}

/** 'SqlTransactionBeforeBlock' query type */
export interface ISqlTransactionBeforeBlockQuery {
  params: ISqlTransactionBeforeBlockParams;
  result: ISqlTransactionBeforeBlockResult;
}

const sqlTransactionBeforeBlockIR: any = {"usedParamSet":{"until_block":true},"params":[{"name":"until_block","required":false,"transform":{"type":"scalar"},"locs":[{"a":98,"b":109}]}],"statement":"WITH block_info AS (\n  SELECT \"Block\".id as until_block_id\n  FROM \"Block\"\n  WHERE \"Block\".hash = (:until_block)\n)\nSELECT \"Transaction\".id\nFROM \"Transaction\"\nWHERE \"Transaction\".block_id <= (SELECT until_block_id FROM block_info)\n                                                                                  \n                                                                                                       \nORDER BY \"Transaction\".block_id DESC, \"Transaction\".id DESC\nLIMIT 1"};

/**
 * Query generated from SQL:
 * ```
 * WITH block_info AS (
 *   SELECT "Block".id as until_block_id
 *   FROM "Block"
 *   WHERE "Block".hash = (:until_block)
 * )
 * SELECT "Transaction".id
 * FROM "Transaction"
 * WHERE "Transaction".block_id <= (SELECT until_block_id FROM block_info)
 *                                                                                   
 *                                                                                                        
 * ORDER BY "Transaction".block_id DESC, "Transaction".id DESC
 * LIMIT 1
 * ```
 */
export const sqlTransactionBeforeBlock = new PreparedQuery<ISqlTransactionBeforeBlockParams,ISqlTransactionBeforeBlockResult>(sqlTransactionBeforeBlockIR);


