/**
 * @example "asset1c43p68zwjezc7f6w4w9qkhkwv9ppwz0f7c3amw"
 */
export type Cip14Fingerprint = string;

export type AssetUtxosRequest = {
  range: { minSlot: number; maxSlot: number },
  assets: Cip14Fingerprint[]
};

export type AssetUtxosResponse = {

    /**
     * If the utxo is created, this has the amount. It's undefined if the utxo
     * is spent. 
     *
     * @example 1031423725351 
     */
    amount: number | undefined,
    utxo: {
      tx: string,
      index: number,
    },
    cip14Fingerprint: string,
    paymentCred: string,
    slot: number
    txId: string,
}[];
