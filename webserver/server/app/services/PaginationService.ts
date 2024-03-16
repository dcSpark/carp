import type { PoolClient } from 'pg';
import { pageStartByHash } from '../models/pagination/pageStartByHash.queries';
import { sqlBlockByHash } from '../models/pagination/sqlBlockByHash.queries';
import { sqlTransactionBeforeBlock } from '../models/pagination/sqlTransactionBeforeBlock.queries';
import type { ISlotBoundsPaginationResult } from '../models/pagination/slotBoundsPagination.queries';

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

export type SlotLimits = {
  // this is exclusive
  from: number;
  // this is inclusive
  to: number;
};

export function adjustToSlotLimits(
  pageStartWithSlot: { tx_id: number, block_id: number } | undefined,
  until: { tx_id: number },
  slotLimits: SlotLimits | undefined,
  slotBounds: ISlotBoundsPaginationResult[] | undefined,
): { tx_id: number, block_id: number } | undefined {
  // if the slotLimits field is set, this shrinks the tx id range
  // accordingly if necessary.
  if (slotLimits) {
    const bounds = slotBounds ? slotBounds[0] : { min_tx_id: -1, max_tx_id: -2 };

    const minTxId = Number(bounds.min_tx_id);

    if (!pageStartWithSlot) {
      pageStartWithSlot = {
        block_id: -1,
        // if no *after* argument is provided, this starts the pagination
        // from the corresponding slot. This allows skipping slots you are
        // not interested in. If there is also no slotLimits specified this
        // starts from the first tx because of the default of -1.
        tx_id: minTxId,
      };
    } else {
      pageStartWithSlot.tx_id = Math.max(Number(bounds.min_tx_id), pageStartWithSlot.tx_id);
    }

    until.tx_id = Math.min(until.tx_id, Number(bounds.max_tx_id));
  }

  return pageStartWithSlot;
}