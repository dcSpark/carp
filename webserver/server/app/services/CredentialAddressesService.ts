import type { PoolClient } from 'pg';
import type { PaginationType } from './PaginationService';
import type { OffsetPaginationRequest } from '../../../shared/models/common';
import { sqlCredentialUsed } from '../models/address/sqlCredentialUsed.queries';
import type { CredentialAddressResponse } from '../../../shared/models/CredentialAddress';

export async function addressesForCredential(
  request: PaginationType['until'] &
    OffsetPaginationRequest & {
      dbTx: PoolClient;
      credentials: Buffer[];
    }
): Promise<CredentialAddressResponse> {
  if (request.credentials.length === 0) return { addresses: [], hasNextPage: false };
  const credentials = await sqlCredentialUsed.run(
    {
      credentials: request.stakeCredentials,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      until_tx_id: request.until.tx_id.toString(),
      relation: request.relationFilter,
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
