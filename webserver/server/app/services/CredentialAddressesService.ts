import type { PoolClient } from 'pg';
import type { PaginationType } from './PaginationService';
import type { ISqlCredentialAddressesResult } from '../models/credentials/sqlCredentialAddresses.queries';
import { sqlCredentialAddresses } from '../models/credentials/sqlCredentialAddresses.queries';
import type { CredentialAddressResponse } from '../../../shared/models/CredentialAddress';
import { cursorFromTxId } from '../models/pagination/cursorFromTxId.queries.queries';
import cml from '@dcspark/cardano-multiplatform-lib-nodejs';
import { CREDENTIAL_LIMIT } from '../../../shared/constants';

export async function addressesForCredential(
  request: PaginationType & {
    dbTx: PoolClient;
    credentials: Buffer[];
  }
): Promise<CredentialAddressResponse> {
  if (request.credentials.length === 0)
    return { addresses: [], pageInfo: { hasNextPage: false, endCursor: undefined } };

  // we fetch an extra result so that we know whether or not a next page exists
  const { addresses, hasNextPage } = await getAddressesAndPage(request);
  const endCursor = await (async () => {
    if (addresses.length === 0) return undefined;
    const pair = (
      await cursorFromTxId.run(
        {
          tx_id: addresses[addresses.length - 1].first_tx,
        },
        request.dbTx
      )
    )[0];
    return {
      tx: pair.tx_hash.toString('hex'),
      block: pair.tx_hash.toString('hex'),
      address: cml.Address.from_bytes(addresses[addresses.length - 1].payload).to_bech32(),
    };
  })();

  return {
    addresses: addresses.map(addr => cml.Address.from_bytes(addr.payload).to_bech32()),
    pageInfo: { hasNextPage, endCursor },
  };
}

async function getAddressesAndPage(
  request: PaginationType & {
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
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
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
