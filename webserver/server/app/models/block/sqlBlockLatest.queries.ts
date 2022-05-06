/** Types generated for queries found in "app/models/block/sqlBlockLatest.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlBlockLatest' parameters type */
export interface ISqlBlockLatestParams {
  offset: string | null | void;
}

/** 'SqlBlockLatest' return type */
export interface ISqlBlockLatestResult {
  epoch: number;
  era: number;
  hash: Buffer;
  height: number;
  id: number;
  slot: number;
}

/** 'SqlBlockLatest' query type */
export interface ISqlBlockLatestQuery {
  params: ISqlBlockLatestParams;
  result: ISqlBlockLatestResult;
}

const sqlBlockLatestIR: any = {"name":"sqlBlockLatest","params":[{"name":"offset","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":91,"b":96,"line":2,"col":64}]}}],"usedParamSet":{"offset":true},"statement":{"body":"SELECT * FROM \"Block\" ORDER BY \"Block\".id DESC LIMIT 1 OFFSET (:offset)","loc":{"a":27,"b":97,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT * FROM "Block" ORDER BY "Block".id DESC LIMIT 1 OFFSET (:offset)
 * ```
 */
export const sqlBlockLatest = new PreparedQuery<ISqlBlockLatestParams,ISqlBlockLatestResult>(sqlBlockLatestIR);


