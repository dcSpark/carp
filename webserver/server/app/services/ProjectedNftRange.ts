import type { PoolClient } from 'pg';
import type { ISqlProjectedNftRangeResult} from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRange } from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRangeByAddress } from '../models/projected_nft/projectedNftRangeByAddress.queries';

export async function projectedNftRange(request: {
    params: { afterSlot: number, untilSlot: number, limit: number },
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRange.run({
        min_slot: request.params.afterSlot,
        max_slot: request.params.untilSlot,
        limit: request.params.limit.toString(),
    }, request.dbTx));
}

export async function projectedNftRangeByAddress(request: {
    address: string,
    params: { afterSlot: number, untilSlot: number, limit: number },
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRangeByAddress.run({
        owner_address: request.address,
        min_slot: request.params.afterSlot,
        max_slot: request.params.untilSlot,
        limit: request.params.limit.toString(),
    }, request.dbTx));
}
