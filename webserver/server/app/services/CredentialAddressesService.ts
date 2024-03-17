import type { PoolClient } from 'pg';
import type { AddressPaginationType } from './PaginationService';
import type { ISqlCredentialAddressesResult } from '../models/credentials/sqlCredentialAddresses.queries';
import { sqlCredentialAddresses } from '../models/credentials/sqlCredentialAddresses.queries';
import type { CredentialAddressResponse } from '../../../shared/models/CredentialAddress';
import cml from '@dcspark/cardano-multiplatform-lib-nodejs';
import { CREDENTIAL_LIMIT } from '../../../shared/constants';

export async function addressesForCredential(
  request: AddressPaginationType & {
    dbTx: PoolClient;
    credentials: Buffer[];
  }
): Promise<CredentialAddressResponse> {
  if (request.credentials.length === 0) return { addresses: [], pageInfo: { hasNextPage: false } };

  // we fetch an extra result so that we know whether or not a next page exists
  const { addresses, hasNextPage } = await getAddressesAndPage(request);

  return {
    addresses: addresses.map(addr => {
      const wasmAddr = cml.Address.from_raw_bytes(addr.payload);
      const bech32 = wasmAddr.to_bech32();
      return bech32;
    }),
    pageInfo: { hasNextPage },
  };
}

async function getAddressesAndPage(
  request: AddressPaginationType & {
    dbTx: PoolClient;
    credentials: Buffer[];
  }
): Promise<{ hasNextPage: boolean; addresses: ISqlCredentialAddressesResult[] }> {
  const adjustedLimit = CREDENTIAL_LIMIT.RESPONSE + 1;
  const sqlResult = await sqlCredentialAddresses.run(
    {
      limit: adjustedLimit.toString(),
      double_limit: (adjustedLimit * 2).toString(),
      credentials: request.credentials,
      after_address: request.after?.address ?? null,
      until_tx_id: request.until.tx_id.toString(),
    },
    request.dbTx
  );
  if (sqlResult.length > CREDENTIAL_LIMIT.RESPONSE) {
    return {
      hasNextPage: true,
      addresses: sqlResult.slice(0, -1),
    };
  } else {
    return {
      hasNextPage: false,
      addresses: sqlResult,
    };
  }
}
