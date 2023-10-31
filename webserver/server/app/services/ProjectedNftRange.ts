import type { PoolClient } from 'pg';
import type { ISqlProjectedNftRangeResult} from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRange } from '../models/projected_nft/projectedNftRange.queries';

export async function projectedNftRange(request: {
    range: { minSlot: number, maxSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRange.run({
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
    }, request.dbTx));
}
