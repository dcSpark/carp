import type { PoolClient } from 'pg';
import { ISqlDrepStakeDelegationForAddressResult, sqlDrepStakeDelegationForAddress } from '../models/delegation/drepDelegationForAddress.queries';

export async function drepDelegationForAddress(request: {
    address: Buffer,
    until: { absoluteSlot: number },
    dbTx: PoolClient,
}): Promise<ISqlDrepStakeDelegationForAddressResult> {
    return (await sqlDrepStakeDelegationForAddress.run({ credential: request.address, slot: request.until.absoluteSlot }, request.dbTx))[0];
}