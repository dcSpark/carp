/** Types generated for queries found in "app/models/pagination/cursorFromTxId.queries.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'CursorFromTxId' parameters type */
export interface ICursorFromTxIdParams {
  tx_id: string | null | void;
}

/** 'CursorFromTxId' return type */
export interface ICursorFromTxIdResult {
  block_hash: Buffer;
  tx_hash: Buffer;
}

/** 'CursorFromTxId' query type */
export interface ICursorFromTxIdQuery {
  params: ICursorFromTxIdParams;
  result: ICursorFromTxIdResult;
}

const cursorFromTxIdIR: any = {"name":"cursorFromTxId","params":[{"name":"tx_id","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":196,"b":200,"line":5,"col":27}]}}],"usedParamSet":{"tx_id":true},"statement":{"body":"SELECT \"Transaction\".hash as tx_hash, \"Block\".hash as block_hash\nFROM \"Transaction\"\nINNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \"Transaction\".id = (:tx_id)","loc":{"a":27,"b":201,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT "Transaction".hash as tx_hash, "Block".hash as block_hash
 * FROM "Transaction"
 * INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE "Transaction".id = (:tx_id)
 * ```
 */
export const cursorFromTxId = new PreparedQuery<ICursorFromTxIdParams,ICursorFromTxIdResult>(cursorFromTxIdIR);


