import { AssetName, PolicyId } from "./PolicyIdAssetMap";
import { Pagination, SlotLimits } from "./common";

/**
 * @example "asset1c43p68zwjezc7f6w4w9qkhkwv9ppwz0f7c3amw"
 */
export type Cip14Fingerprint = string;

export type AssetUtxosRequest = {
  fingerprints?: Cip14Fingerprint[];
  policyIds?: PolicyId[];
  /** This limits the transactions in the result to this range of slots.
   * Everything else is filtered out */
  slotLimits?: SlotLimits;

  limit?: number;
} & Pagination;

export type AssetUtxosResponse = {
  payload: {
    /**
     * If the utxo is created, this has the amount. It's undefined if the utxo
     * is spent.
     *
     * @example '1031423725351'
     */
    amount: string | undefined;
    cip14Fingerprint: string;
    policyId: string;
    assetName: AssetName;
    paymentCred: string;
    slot: number;
    utxo: {
      tx: string;
      index: number;
    };
  }[];
  txId: string;
  block: string;
}[];
