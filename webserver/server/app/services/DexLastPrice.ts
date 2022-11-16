import { Asset, PriceType, DexLastPriceResponse } from '../../../shared/models/DexLastPrice';
import type { PoolClient } from 'pg';
import { sqlDexLastPriceSwap } from '../models/dex/sqlDexLastPriceSwap.queries';
import { sqlDexLastPriceMean } from '../models/dex/sqlDexLastPriceMean.queries';
import { parseAssetItem, serializeAsset} from './utils';


export async function dexLastPrice(
  request: {
    dbTx: PoolClient;
    assetPairs: {asset1: Asset, asset2: Asset}[];
    type: PriceType;
  }
): Promise<DexLastPriceResponse> {
  if (request.assetPairs.length === 0) return { lastPrice: [] };

  
    const lastPrice = await (async () => {
        switch(request.type) {
            case PriceType.Mean:
                return await sqlDexLastPriceMean.run({
                    policy_id1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.policyId)),
                    asset_name1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.assetName)),
                    policy_id2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.policyId)),
                    asset_name2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.assetName)),
                }, request.dbTx);

            case PriceType.Sell:
                return await sqlDexLastPriceSwap.run({
                    policy_id1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.policyId)),
                    asset_name1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.assetName)),
                    policy_id2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.policyId)),
                    asset_name2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.assetName)),
                    direction: false
                }, request.dbTx);

            case PriceType.Buy:
                return await sqlDexLastPriceSwap.run({
                    policy_id1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.policyId)),
                    asset_name1: request.assetPairs.map(pair => parseAssetItem(pair.asset1?.assetName)),
                    policy_id2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.policyId)),
                    asset_name2: request.assetPairs.map(pair => parseAssetItem(pair.asset2?.assetName)),
                    direction: true
                }, request.dbTx);
        }
    })();

  return {
    lastPrice: lastPrice.map(result => ({
      asset1: serializeAsset(result.policy_id1, result.asset_name1),
      asset2: serializeAsset(result.policy_id2, result.asset_name2),
      amount1: result.amount1,
      amount2: result.amount2,
      dex: String(result.dex),
    })),
  };
}
