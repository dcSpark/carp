/** Types generated for queries found in "app/models/block/sqlBlockLatest.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type NumberOrString = number | string;

/** 'SqlBlockLatest' parameters type */
export interface ISqlBlockLatestParams {
  offset?: NumberOrString | null | void;
}

/** 'SqlBlockLatest' return type */
export interface ISqlBlockLatestResult {
  epoch: number;
  era: number;
  hash: Buffer;
  height: number;
  id: number;
  payload: Buffer | null;
  slot: number;
}

/** 'SqlBlockLatest' query type */
export interface ISqlBlockLatestQuery {
  params: ISqlBlockLatestParams;
  result: ISqlBlockLatestResult;
}

const sqlBlockLatestIR: any = {"usedParamSet":{"offset":true},"params":[{"name":"offset","required":false,"transform":{"type":"scalar"},"locs":[{"a":63,"b":69}]}],"statement":"SELECT * FROM \"Block\" ORDER BY \"Block\".id DESC LIMIT 1 OFFSET (:offset)"};

/**
 * Query generated from SQL:
 * ```
 * SELECT * FROM "Block" ORDER BY "Block".id DESC LIMIT 1 OFFSET (:offset)
 * ```
 */
export const sqlBlockLatest = new PreparedQuery<ISqlBlockLatestParams,ISqlBlockLatestResult>(sqlBlockLatestIR);


