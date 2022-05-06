import type { AddressUsedResponse } from '../../../shared/models/AddressUsed';
import type { PoolClient } from 'pg';
import type { PaginationType } from './PaginationService';
import type { RelationFilter } from '../../../shared/models/common';
import { sqlAddressUsed } from '../models/address/sqlAddressUsed.queries';
import { sqlCredentialUsed } from '../models/address/sqlCredentialUsed.queries';

export async function credentialUsed(
  request: PaginationType & {
    dbTx: PoolClient;
    stakeCredentials: Buffer[];
    relationFilter: RelationFilter;
    reverseMap: Map<string, Set<string>>;
  }
): Promise<AddressUsedResponse> {
  if (request.stakeCredentials.length === 0) return { addresses: [] };
  const credenials = await sqlCredentialUsed.run(
    {
      credentials: request.stakeCredentials,
      after_block_id: request.after?.block_id ?? -1,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      until_block_id: request.until.block_id,
      relation: request.relationFilter,
    },
    request.dbTx
  );
  const usedCredHex = credenials.map(cred => cred.credential.toString('hex'));
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
      after_block_id: request.after?.block_id ?? -1,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      until_block_id: request.until.block_id,
    },
    request.dbTx
  );
  const usedAddrHex = addresses.map(address => Buffer.from(address.payload).toString('hex'));
  const result = new Set<string>();
  for (const addrHex of usedAddrHex) {
    (request.reverseMap.get(addrHex) ?? []).forEach(addr => result.add(addr));
  }
  return {
    addresses: Array.from(result),
  };
}
