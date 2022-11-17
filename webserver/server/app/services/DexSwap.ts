import type { Asset, DexSwapResponse } from '../../../shared/models/DexSwap';
import type { PoolClient } from 'pg';
import type { TransactionPaginationType } from './PaginationService';
import type { Dex } from '../../../shared/models/common';
import { sqlDexSwap } from '../models/dex/sqlDexSwap.queries';
import { parseAssetItem, serializeAsset, valueToDex, dexToValue} from './utils';
import { Direction } from '../../../shared/models/common';

export async function dexSwap(
  request: TransactionPaginationType & {
    dbTx: PoolClient;
    dexes: Array<Dex>;
    assetPairs: {asset1: Asset, asset2: Asset}[];
    limit: number;
  }
): Promise<DexSwapResponse> {
  if (request.assetPairs.length === 0) return { swap: [] };

  const swap = await sqlDexSwap.run({
    after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
    until_tx_id: request.until.tx_id.toString(),
    dexes: request.dexes.map(dex => dexToValue(dex)),
    policy_id1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.policyId)),
    asset_name1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.assetName)),
    policy_id2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.policyId)),
    asset_name2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.assetName)),
    limit: request.limit.toString(),
  }, request.dbTx);
  return {
    swap: swap.map(result => ({
      tx_hash: result.tx_hash.toString('hex'),
      dex: valueToDex(result.dex),
      asset1: serializeAsset(result.policy_id1, result.asset_name1),
      asset2: serializeAsset(result.policy_id2, result.asset_name2),
      amount1: result.amount1,
      amount2: result.amount2,
      direction: (result.direction ? Direction.Buy : Direction.Sell)
    })),
  };
}
