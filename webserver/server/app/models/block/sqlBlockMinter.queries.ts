/** Types generated for queries found in "app/models/block/sqlBlockMinter.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'SqlBlockMinter' parameters type */
export interface ISqlBlockMinterParams {
  addresses: BufferArray | null | void;
}

/** 'SqlBlockMinter' return type */
export interface ISqlBlockMinterResult {
  key: Buffer;
}

/** 'SqlBlockMinter' query type */
export interface ISqlBlockMinterQuery {
  params: ISqlBlockMinterParams;
  result: ISqlBlockMinterResult;
}

const sqlBlockMinterIR: any = {"name":"sqlBlockMinter","params":[{"name":"addresses","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":136,"b":144,"line":4,"col":27}]}}],"usedParamSet":{"addresses":true},"statement":{"body":"SELECT key FROM \"Block\"\nINNER JOIN \"BlockMinter\" ON \"BlockMinter\".id = \"Block\".id\nWHERE \"Block\".hash = ANY (:addresses)","loc":{"a":27,"b":145,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT key FROM "Block"
 * INNER JOIN "BlockMinter" ON "BlockMinter".id = "Block".id
 * WHERE "Block".hash = ANY (:addresses)
 * ```
 */
export const sqlBlockMinter = new PreparedQuery<ISqlBlockMinterParams,ISqlBlockMinterResult>(sqlBlockMinterIR);


