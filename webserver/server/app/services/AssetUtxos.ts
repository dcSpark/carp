import type { PoolClient } from 'pg';
import type { IAssetUtxosResult } from '../models/asset/assetUtxos.queries';
import { assetUtxos } from '../models/asset/assetUtxos.queries';

export async function getAssetUtxos(request: {
  range: {
    minSlot: number;
    maxSlot: number;
  };
  assets: Buffer[];
  dbTx: PoolClient;
}): Promise<IAssetUtxosResult[]> {
  return await assetUtxos.run(
    {
      max_slot: request.range.maxSlot,
      min_slot: request.range.minSlot,
      fingerprints: request.assets,
    },
    request.dbTx
  );
}
