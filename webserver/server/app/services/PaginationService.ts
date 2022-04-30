import type { PoolClient } from 'pg';
import { pageStartByHash } from '../models/pageStartByHash.queries';
import { sqlBlockByHash } from '../models/sqlBlockByHash.queries';

export type PaginationType = {
  after:
    | undefined
    | {
        block_id: number;
        tx_id: number;
      };
  until: {
    block_id: number;
  };
  limit: number;
};

export async function resolveUntilBlock(request: {
  block_hash: Buffer;
  dbTx: PoolClient;
}): Promise<undefined | PaginationType['until']> {
  const result = await sqlBlockByHash.run(
    {
      until_block: request.block_hash,
    },
    request.dbTx
  );
  if (result[0] == null) return undefined;
  return {
    block_id: result[0].until_block_id,
  };
}

export async function resolvePageStart(request: {
  after_block: Buffer;
  after_tx: Buffer;
  dbTx: PoolClient;
}): Promise<undefined | PaginationType['after']> {
  const result = await pageStartByHash.run(
    {
      after_block: request.after_block,
      after_tx: request.after_tx,
    },
    request.dbTx
  );
  if (result[0] == null) return undefined;
  return {
    block_id: result[0].after_block_id,
    tx_id: Number.parseInt(result[0].after_tx_id, 10),
  };
}
