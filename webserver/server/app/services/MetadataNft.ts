import type { Pool } from 'pg';
import { sqlMetadataNft } from '../models/metadata/sqlMetadataNft.queries';
import type { Cip25Response, NativeAsset } from '../../../shared/models/PolicyIdAssetMap';

export async function metadataNfts(request: {
  pairs: NativeAsset[];
  dbTx: Pool;
}): Promise<Cip25Response> {
  if (request.pairs.length === 0) return { cip25: {} };
  const cip25 = await sqlMetadataNft.run(
    {
      policy_id: request.pairs.map(pair => Buffer.from(pair[0], 'hex')),
      asset_name: request.pairs.map(pair => Buffer.from(pair[1], 'hex')),
    },
    request.dbTx
  );
  const result: Cip25Response['cip25'] = {};
  for (const entry of cip25) {
    const policy_hex = entry.policy_id.toString('hex');
    const for_policy = result[policy_hex] ?? {};

    for_policy[entry.asset_name.toString('hex')] = entry.payload.toString('hex');
    result[policy_hex] = for_policy;
  }
  return { cip25: result };
}
