/** Types generated for queries found in "app/models/pagination/pageStartByHash.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'PageStartByHash' parameters type */
export interface IPageStartByHashParams {
  after_block?: Buffer | null | void;
  after_tx?: Buffer | null | void;
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

const pageStartByHashIR: any = {"usedParamSet":{"after_block":true,"after_tx":true},"params":[{"name":"after_block","required":false,"transform":{"type":"scalar"},"locs":[{"a":174,"b":185}]},{"name":"after_tx","required":false,"transform":{"type":"scalar"},"locs":[{"a":219,"b":227}]}],"statement":"SELECT\n  \"Block\".id as after_block_id,\n  \"Transaction\".id as after_tx_id\nFROM \"Transaction\" INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n  \"Block\".hash = (:after_block)\n  AND \n  \"Transaction\".hash = (:after_tx)"};

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


