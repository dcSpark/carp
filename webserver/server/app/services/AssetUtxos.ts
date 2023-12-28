import type { PoolClient } from 'pg';
import type { IAssetUtxosResult } from '../models/asset/assetUtxos.queries';
import { assetUtxos } from '../models/asset/assetUtxos.queries';

export async function getAssetUtxos(request: {
  range: {
    minSlot: number;
    maxSlot: number;
  };
  fingerprints?: Buffer[];
  policyIds?: Buffer[];
  dbTx: PoolClient;
}): Promise<IAssetUtxosResult[]> {
  return await assetUtxos.run(
    {
      max_slot: request.range.maxSlot,
      min_slot: request.range.minSlot,
      // pgtyped doesn't seem to have a way to have an optional array parameter,
      // and an empty spread expansion fails with postgres.  Since none of these
      // fields is nullable, an array with null should be equivalent to an empty
      // array.
      fingerprints:
        request.fingerprints && request.fingerprints.length > 0 ? request.fingerprints : [null],
      policyIds: request.policyIds && request.policyIds.length > 0 ? request.policyIds : [null],
    },
    request.dbTx
  );
}
