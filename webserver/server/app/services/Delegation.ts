
import type { PoolClient } from 'pg';
import { ISqlStakeDelegationResult, sqlStakeDelegation } from '../models/delegation/delegationForAddress.queries';


export async function delegationForAddress(request: {
    address: Buffer,
    until: { absoluteSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlStakeDelegationResult> {
    return (await sqlStakeDelegation.run({ credential: request.address, slot: request.until.absoluteSlot }, request.dbTx))[0];
}