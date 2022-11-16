import type { Asset } from '../../../shared/models/DexMeanPrice';

export function parseAssetItem(s: string | undefined | null): Buffer {
    // For the sake of the query, we represent ADA as ('', '') instead of (NULL, NULL).
    // (see sqlDexMeanPrice.queries.sql for details)
    return Buffer.from(s ?? "", 'hex');
  }
  
  export function serializeAsset(policyId: Buffer | null, assetName: Buffer | null): Asset {
    if (policyId === null && assetName === null) {
      return null;
    }
    if (policyId !== null && assetName !== null) {
      return {
        policyId: policyId.toString('hex'),
        assetName: assetName.toString('hex'),
      };
    }
    throw new Error('Invalid asset query response'); // should be unreachable
  }
  