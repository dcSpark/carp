import { Pagination, SlotLimits } from "./common";

export type ProjectedNftRangeRequest = {
  address: string | undefined;
  slotLimits?: SlotLimits;
  limit?: number;
} & Pagination;

export enum ProjectedNftStatus {
  Lock = "Lock",
  Unlocking = "Unlocking",
  Claim = "Claim",
  Invalid = "Invalid",
}

export type ProjectedNftRangeResponse = {
  payload: {
    /**
     * Slot at which the transaction happened
     *
     * @example 512345
     */
    actionSlot: number;

    /**
     * Projected NFT owner address. Not null only if owned by Public Key Hash.
     *
     * @pattern [0-9a-fA-F]+
     * @example "9040f057461d9adc09108fe5cb630077cf75c6e981d3ed91f6fb18f6"
     */
    ownerAddress: string | null;

    /**
     * Output index of related Projected NFT event. Null if it is claim event (No new UTxO is created).
     *
     * @example 1
     */
    actionOutputIndex: number | null;

    /**
     * Transaction id of related previous Projected NFT event.
     * E.g. you locked the NFT and get unlocking event: you will see previousTxHash = transaction hash of lock event.
     * Null if event has `status` Lock.
     *
     * @pattern [0-9a-fA-F]{64}
     * @example "28eb069e3e8c13831d431e3b2e35f58525493ab2d77fde83184993e4aa7a0eda"
     */
    previousTxHash: string | null;
    /**
     * Output index of related previous Projected NFT event. Null if event has `status` Lock.
     *
     * @example 1
     */
    previousTxOutputIndex: number | null;
    /**
     * Asset policy id that relates to Projected NFT event
     *
     * @pattern [0-9a-fA-F]{56}
     * @example "96f7dc9749ede0140f042516f4b723d7261610d6b12ccb19f3475278"
     */
    policyId: string;
    /**
     * Asset name that relates to Projected NFT event
     *
     * @pattern ([0-9a-fA-F]{2}){0,32}
     * @example "415045"
     */
    assetName: string;
    /**
     * Number of assets of `asset` type used in this Projected NFT event.
     *
     * @example "1"
     */
    amount: string;
    /**
     * Projected NFT status: Lock / Unlocking / Claim / Invalid
     *
     * @example "Lock"
     */
    status: ProjectedNftStatus;
    /**
     * Projected NFT datum: serialized state of the Projected NFT
     *
     * @pattern [0-9a-fA-F]+
     * @example "d8799fd8799f581c9040f057461d9adc09108fe5cb630077cf75c6e981d3ed91f6fb18f6ffd87980ff"
     */
    plutusDatum: string;
    /**
     * UNIX timestamp till which the funds can't be claimed in the Unlocking state.
     * If the status is not Unlocking this is always null.
     *
     * @example "1701266986000"
     */
    forHowLong: string | null;
  }[];
  block: string;

  /**
   * Transaction id of related Projected NFT event
   *
   * @pattern [0-9a-fA-F]{64}
   * @example "28eb069e3e8c13831d431e3b2e35f58525493ab2d77fde83184993e4aa7a0eda"
   */
  txId: string;
}[];
