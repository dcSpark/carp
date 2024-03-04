import type { PoolClient } from 'pg';
import type { ISqlMintBurnRangeResult, ISqlMintBurnRangeByPolicyIdsResult } from '../models/asset/mintBurnHistory.queries';
import { sqlMintBurnRange, sqlMintBurnRangeByPolicyIds } from '../models/asset/mintBurnHistory.queries';
import type { PolicyId } from "../../../shared/models/PolicyIdAssetMap";

export async function mintBurnRange(request: {
    range: { minSlot: number, maxSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlMintBurnRangeResult[]> {
    return (await sqlMintBurnRange.run({
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
    }, request.dbTx));
}

export async function mintBurnRangeByPolicyIds(request: {
    range: { minSlot: number, maxSlot: number },
    policyIds: PolicyId[],
    dbTx: PoolClient,
}): Promise<ISqlMintBurnRangeByPolicyIdsResult[]> {
    return (await sqlMintBurnRangeByPolicyIds.run({
        min_slot: request.range.minSlot,
        max_slot: request.range.maxSlot,
        policy_ids: request.policyIds.map(id => Buffer.from(id, 'hex')),
    }, request.dbTx));
}
