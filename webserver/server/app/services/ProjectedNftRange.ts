import type { PoolClient } from 'pg';
import type { ISqlProjectedNftRangeResult} from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRange } from '../models/projected_nft/projectedNftRange.queries';
import { sqlProjectedNftRangeByAddress } from '../models/projected_nft/projectedNftRangeByAddress.queries';

export async function projectedNftRange(request: {
    after: number;
    until: number;
    limit: number;
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRange.run({
        after_tx_id: request.after,
        until_tx_id: request.until,
        limit: request.limit,
    }, request.dbTx));
}

export async function projectedNftRangeByAddress(request: {
    address: string,
    after: number;
    until: number;
    limit: number;
    dbTx: PoolClient,
}): Promise<ISqlProjectedNftRangeResult[]> {
    return (await sqlProjectedNftRangeByAddress.run({
        owner_address: request.address,
        after_tx_id: request.after,
        until_tx_id: request.until,
        limit: request.limit,
    }, request.dbTx));
}
