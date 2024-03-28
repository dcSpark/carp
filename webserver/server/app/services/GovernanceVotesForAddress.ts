import type { PoolClient } from 'pg';
import {
  IVotesForAddressResult,
  votesForAddress,
} from '../models/governance/votesForAddress.queries';

export async function governanceVotesForAddress(request: {
  address: Buffer;
  dbTx: PoolClient;
  limit: number;
  before: number;
  until: number;
}): Promise<IVotesForAddressResult[]> {
  return (
    await votesForAddress.run(
      {
        voter: request.address,
        limit: request.limit,
        before_tx_id: request.before,
        until_tx_id: request.until,
      },
      request.dbTx
    )
  );
}
