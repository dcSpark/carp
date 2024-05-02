import type { PoolClient } from 'pg';
import {
    didVote,
  IDidVoteResult,
} from '../models/governance/votesForAddress.queries';

export async function governanceCredentialDidVote(request: {
  credential: Buffer;
  govActionIds: Buffer[];
  dbTx: PoolClient;
  until: number;
}): Promise<IDidVoteResult[]> {
  return (
    await didVote.run(
      {
        voter: request.credential,
        gov_action_ids: request.govActionIds,
        until_tx_id: request.until,
      },
      request.dbTx
    )
  );
}

