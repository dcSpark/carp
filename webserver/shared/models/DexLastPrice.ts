import { BlockSubset } from "./BlockLatest";
import { Dex, Asset, Amount } from "./common";

export type DexLastPrice = {
  asset1: Asset;
  asset2: Asset;
  amount1: Amount;
  amount2: Amount;
  dex: Dex;
  block: Omit<BlockSubset, "era">;
};

export enum PriceType {
  Buy = "buy",
  Sell = "sell",
  /**
   * Mean is not AVG from the last values, but the remaining amount of assets on the pool output
   */
  Mean = "mean",
}

export type DexLastPriceRequest = {
  assetPairs: { asset1: Asset; asset2: Asset }[];
  type: PriceType;
};

export type DexLastPriceResponse = {
  lastPrice: DexLastPrice[];
};
