/** Types generated for queries found in "app/models/pageStartByHash.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'PageStartByHash' parameters type */
export interface IPageStartByHashParams {
  after_block: Buffer | null | void;
  after_tx: Buffer | null | void;
}

/** 'PageStartByHash' return type */
export interface IPageStartByHashResult {
  after_block_id: number;
  after_tx_id: string;
}

/** 'PageStartByHash' query type */
export interface IPageStartByHashQuery {
  params: IPageStartByHashParams;
  result: IPageStartByHashResult;
}

const pageStartByHashIR: any = {"name":"pageStartByHash","params":[{"name":"after_block","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":203,"b":213,"line":7,"col":19}]}},{"name":"after_tx","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":248,"b":255,"line":9,"col":25}]}}],"usedParamSet":{"after_block":true,"after_tx":true},"statement":{"body":"SELECT\n  \"Block\".id as after_block_id,\n  \"Transaction\".id as after_tx_id\nFROM \"Transaction\" INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n  \"Block\".hash = (:after_block)\n  AND \n  \"Transaction\".hash = (:after_tx)","loc":{"a":28,"b":256,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *   "Block".id as after_block_id,
 *   "Transaction".id as after_tx_id
 * FROM "Transaction" INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *   "Block".hash = (:after_block)
 *   AND 
 *   "Transaction".hash = (:after_tx)
 * ```
 */
export const pageStartByHash = new PreparedQuery<IPageStartByHashParams,IPageStartByHashResult>(pageStartByHashIR);


