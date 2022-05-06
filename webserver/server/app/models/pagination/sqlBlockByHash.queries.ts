/** Types generated for queries found in "app/models/pagination/sqlBlockByHash.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlBlockByHash' parameters type */
export interface ISqlBlockByHashParams {
  until_block: Buffer | null | void;
}

/** 'SqlBlockByHash' return type */
export interface ISqlBlockByHashResult {
  until_block_id: number;
}

/** 'SqlBlockByHash' query type */
export interface ISqlBlockByHashQuery {
  params: ISqlBlockByHashParams;
  result: ISqlBlockByHashResult;
}

const sqlBlockByHashIR: any = {"name":"sqlBlockByHash","params":[{"name":"until_block","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":99,"b":109,"line":4,"col":23}]}}],"usedParamSet":{"until_block":true},"statement":{"body":"SELECT \"Block\".id as until_block_id\nFROM \"Block\"\nWHERE \"Block\".hash = (:until_block)","loc":{"a":27,"b":110,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT "Block".id as until_block_id
 * FROM "Block"
 * WHERE "Block".hash = (:until_block)
 * ```
 */
export const sqlBlockByHash = new PreparedQuery<ISqlBlockByHashParams,ISqlBlockByHashResult>(sqlBlockByHashIR);


