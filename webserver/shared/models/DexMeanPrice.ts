import { Dex, Pagination, Asset } from "./common";

/**
 * @example "2042352568679"
 */
type Amount = string; // uint64

export type DexMeanPrice = {
    tx_hash: string;
    dex: Dex;
    asset1: Asset;
    asset2: Asset;
    amount1: Amount;
    amount2: Amount;
}

export type DexMeanPriceRequest = {
  assetPairs: {asset1: Asset, asset2: Asset}[];
  dexes: Array<Dex>,
  /** Defaults to `DEX_PRICE_LIMIT.RESPONSE` */
  limit?: number;
} & Pagination;

export type DexMeanPriceResponse = {
  meanPrices: DexMeanPrice[];
};
