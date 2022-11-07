import type { Pool } from 'pg';
import type { BlockMinterRequest, BlockMinterResponse } from '../../../shared/models/BlockMinter';
// import { sqlBlockMinter } from '../models/block/sqlBlockMinter.queries';

export async function getBlockMinter(
  request: BlockMinterRequest & { dbTx: Pool }
): Promise<undefined | BlockMinterResponse> {
  return undefined;
  // const bestBlock = await sqlBlockMinter.run(
  //   {
  //     offset: request.offset.toString(),
  //   },
  //   request.dbTx
  // );
  // return {
  //   block: bestBlock.map(block => ({
  //     era: block.era,
  //     hash: block.hash.toString('hex'),
  //     height: block.height,
  //     epoch: block.epoch,
  //     slot: block.slot,
  //   }))[0],
  // };
}
