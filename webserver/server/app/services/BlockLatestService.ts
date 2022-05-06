import type { Pool } from 'pg';
import type { BlockLatestRequest, BlockLatestResponse } from '../../../shared/models/BlockLatest';
import { sqlBlockLatest } from '../models/block/sqlBlockLatest.queries';

export async function getLatestBlock(
  request: BlockLatestRequest & { dbTx: Pool }
): Promise<undefined | BlockLatestResponse> {
  const bestBlock = await sqlBlockLatest.run(
    {
      offset: request.offset.toString(),
    },
    request.dbTx
  );
  return {
    block: bestBlock.map(block => ({
      era: block.era,
      hash: block.hash.toString('hex'),
      height: block.height,
      epoch: block.epoch,
      slot: block.slot,
    }))[0],
  };
}
