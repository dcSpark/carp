import type { PoolClient } from 'pg';
import type { ISqlProjectedNftRangeResult} from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRange } from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRangeByAddress } from '../models/projected_nft/projectedNftRangeByAddress.queries';

export async function projectedNftRange(request: {
    range: { minSlot: number, maxSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRange.run({
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
    }, request.dbTx));
}

export async function projectedNftRangeByAddress(request: {
    address: string,
    range: { minSlot: number, maxSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRangeByAddress.run({
        owner_address: request.address,
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
    }, request.dbTx));
}
