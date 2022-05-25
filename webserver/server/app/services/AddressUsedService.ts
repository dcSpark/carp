import type { AddressUsedResponse } from '../../../shared/models/AddressUsed';
import type { PoolClient } from 'pg';
import type { PaginationType } from './PaginationService';
import { sqlAddressUsed } from '../models/address/sqlAddressUsed.queries';
import { sqlCredentialUsed } from '../models/address/sqlCredentialUsed.queries';

export async function credentialUsed(
  request: PaginationType & {
    dbTx: PoolClient;
    stakeCredentials: Buffer[];
    reverseMap: Map<string, Set<string>>;
  }
): Promise<AddressUsedResponse> {
  if (request.stakeCredentials.length === 0) return { addresses: [] };
  const credentials = await sqlCredentialUsed.run(
    {
      credentials: request.stakeCredentials,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      until_tx_id: request.until.tx_id.toString(),
    },
    request.dbTx
  );
  const usedCredHex = credentials.map(cred => cred.credential.toString('hex'));
  const result = new Set<string>();
  for (const credHex of usedCredHex) {
    (request.reverseMap.get(credHex) ?? []).forEach(addr => result.add(addr));
  }
  return {
    addresses: Array.from(result),
  };
}

export async function addressUsed(
  request: PaginationType & {
    addresses: Buffer[];
    dbTx: PoolClient;
    reverseMap: Map<string, Set<string>>;
  }
): Promise<AddressUsedResponse> {
  if (request.addresses?.length === 0) return { addresses: [] };
  const addresses = await sqlAddressUsed.run(
    {
      addresses: request.addresses,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      until_tx_id: request.until.tx_id.toString(),
    },
    request.dbTx
  );
  const usedAddrHex = addresses.reduce((acc, next) => {
    if (next.payload == null) return acc;
    acc.push(next.payload.toString('hex'));
    return acc;
  }, [] as string[]);
  const result = new Set<string>();
  for (const addrHex of usedAddrHex) {
    (request.reverseMap.get(addrHex) ?? []).forEach(addr => result.add(addr));
  }
  return {
    addresses: Array.from(result),
  };
}
