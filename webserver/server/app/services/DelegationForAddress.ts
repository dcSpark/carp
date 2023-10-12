import type { PoolClient } from 'pg';
import { ISqlStakeDelegationForAddressResult, sqlStakeDelegationForAddress } from '../models/delegation/delegationForAddress.queries';

export async function delegationForAddress(request: {
    address: Buffer,
    until: { absoluteSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlStakeDelegationForAddressResult> {
    return (await sqlStakeDelegationForAddress.run({ credential: request.address, slot: request.until.absoluteSlot }, request.dbTx))[0];
}