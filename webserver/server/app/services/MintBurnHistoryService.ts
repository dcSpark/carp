import type { PoolClient } from 'pg';
import type {
  ISqlMintBurnRangeResult,
  ISqlMintBurnRangeByPolicyIdsResult,
} from '../models/asset/mintBurnHistory.queries';
import {
  sqlMintBurnRange,
  sqlMintBurnRangeByPolicyIds,
} from '../models/asset/mintBurnHistory.queries';
import type { PolicyId } from '../../../shared/models/PolicyIdAssetMap';

export async function mintBurnRange(request: {
  after: number;
  until: number;
  limit: number;
  dbTx: PoolClient;
}): Promise<ISqlMintBurnRangeResult[]> {
  return await sqlMintBurnRange.run(
    {
      after_tx_id: request.after,
      until_tx_id: request.until,
      limit: request.limit,
    },
    request.dbTx
  );
}

export async function mintBurnRangeByPolicyIds(request: {
  after: number;
  until: number;
  limit: number;
  policyIds: PolicyId[];
  dbTx: PoolClient;
}): Promise<ISqlMintBurnRangeByPolicyIdsResult[]> {
  return await sqlMintBurnRangeByPolicyIds.run(
    {
      after_tx_id: request.after,
      until_tx_id: request.until,
      limit: request.limit,
      policy_ids: request.policyIds.map(id => Buffer.from(id, 'hex')),
    },
    request.dbTx
  );
}
