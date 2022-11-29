import { Dex, Asset, Amount } from "./common";

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
