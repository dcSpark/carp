import type { PoolClient } from 'pg';
import { pageStartByHash } from '../models/pagination/pageStartByHash.queries';
import { sqlBlockByHash } from '../models/pagination/sqlBlockByHash.queries';
import { sqlTransactionBeforeBlock } from '../models/pagination/sqlTransactionBeforeBlock.queries';

export type UntilPaginationType = {
  until: {
    tx_id: number;
  };
};
export type AddressPaginationType = UntilPaginationType & {
  after:
    | undefined
    | {
        address: Buffer; // payload of bech32
      };
};
export type TransactionPaginationType = UntilPaginationType & {
  after:
    | undefined
    | {
        block_id: number;
        tx_id: number;
      };
};

export async function resolveUntilTransaction(request: {
  block_hash: Buffer;
  dbTx: PoolClient;
}): Promise<undefined | UntilPaginationType['until']> {
  const result = await sqlTransactionBeforeBlock.run(
    {
      until_block: request.block_hash,
    },
    request.dbTx
  );
  if (result[0] == null) return undefined;
  return {
    tx_id: Number.parseInt(result[0].id, 10),
  };
}

export async function resolveUntilBlock(request: {
  block_hash: Buffer;
  dbTx: PoolClient;
}): Promise<undefined | { block_id: number }> {
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
}): Promise<undefined | TransactionPaginationType['after']> {
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
