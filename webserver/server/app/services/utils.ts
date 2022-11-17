import type { Asset } from '../../../shared/models/common';
import { Dex } from '../../../shared/models/common';

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
  

export function valueToDex(dex: string) {
    switch(dex) {
        case '0': return Dex.WingRiders;
        case '1': return Dex.SundaeSwap;
        case '2': return Dex.MinSwap;
    }
    return Dex.Unknown;
}

export function dexToValue(dex: Dex) {
    switch(dex) {
        case Dex.WingRiders: return '0';
        case Dex.SundaeSwap: return '1';
        case Dex.MinSwap: return '2';
    }
    return '-1';
}