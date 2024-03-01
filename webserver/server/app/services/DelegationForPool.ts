import type { PoolClient } from 'pg';
import type { ISqlStakeDelegationByPoolResult } from '../models/delegation/delegationsForPool.queries';
import { sqlStakeDelegationByPool } from '../models/delegation/delegationsForPool.queries';

export async function delegationsForPool(request: {
  after: number;
  until: number;
  pools: Buffer[];
  limit: number;
  dbTx: PoolClient;
}): Promise<ISqlStakeDelegationByPoolResult[]> {
  return await sqlStakeDelegationByPool.run(
    {
      after_tx_id: request.after,
      until_tx_id: request.until,
      pools: request.pools,
      limit: request.limit,
    },
    request.dbTx
  );
}
