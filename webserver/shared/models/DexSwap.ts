import { Pagination, Dex, Direction } from "./common";
import { AssetName, PolicyId } from "./PolicyIdAssetMap";

export type Asset = {
  policyId: PolicyId;
  assetName: AssetName;
} | null;

/**
 * @example "2042352568679"
 */
type Amount = string; // uint64


export type DexSwap = {
    tx_hash: string;
    dex: Dex;
    asset1: Asset;
    asset2: Asset;
    amount1: Amount;
    amount2: Amount;
    direction: Direction;
}

export type DexSwapRequest = {
  dexes: Array<Dex>,
  assetPairs: {asset1: Asset, asset2: Asset}[];
  /** Defaults to `DEX_PRICE_LIMIT.RESPONSE` */
  limit?: number;
} & Pagination;

export type DexSwapResponse = {
  swap: DexSwap[];
};
