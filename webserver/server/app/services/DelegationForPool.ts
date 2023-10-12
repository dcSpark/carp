import type { PoolClient } from 'pg';
import { ISqlStakeDelegationByPoolResult, sqlStakeDelegationByPool } from '../models/delegation/delegationsForPool.queries';


export async function delegationsForPool(request: {
    range: { minSlot: number, maxSlot: number },
    pools: Buffer[],
    dbTx: PoolClient,
}): Promise<ISqlStakeDelegationByPoolResult[]> {
    return (await sqlStakeDelegationByPool.run({
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
        pools: request.pools
    }, request.dbTx));
}