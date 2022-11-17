import { Dex, Asset } from "./common";

/**
 * @example "2042352568679"
 */
type Amount = string; // uint64

export type DexLastPrice = {
    asset1: Asset;
    asset2: Asset;
    amount1: Amount;
    amount2: Amount;
    dex: Dex;
};

export enum PriceType {
    Buy = "buy",
    Sell = "sell",
    Mean = "mean",
};

export type DexLastPriceRequest = {
  assetPairs: {asset1: Asset, asset2: Asset}[];
  type: PriceType;
};

export type DexLastPriceResponse = {
  lastPrice: DexLastPrice[];
};
