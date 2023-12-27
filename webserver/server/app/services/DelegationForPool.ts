import type { PoolClient } from 'pg';
import type { ISqlStakeDelegationByPoolResult} from '../models/delegation/delegationsForPool.queries';
import { sqlStakeDelegationByPool } from '../models/delegation/delegationsForPool.queries';

export async function delegationsForPool(request: {
    params: { afterSlot: number, untilSlot: number, limit: number },
    pools: Buffer[],
    dbTx: PoolClient,
}): Promise<ISqlStakeDelegationByPoolResult[]> {
    return (await sqlStakeDelegationByPool.run({
        min_slot: request.params.afterSlot,
        max_slot: request.params.untilSlot,
        limit: request.params.limit.toString(),
        pools: request.pools
    }, request.dbTx));
}