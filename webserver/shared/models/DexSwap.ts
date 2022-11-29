import { Pagination, Dex, Direction, Asset, Amount } from "./common";

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
