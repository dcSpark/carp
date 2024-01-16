import {PolicyId} from "./PolicyIdAssetMap"
import {Amount} from "./common";

export type MintBurnHistoryRequest = {
    /**
     * Mint Burn events in this slot range will be returned
     */
    range: {
        /**
         * Minimal slot from which the events should be returned (not inclusive)
         *
         * @example 46154769
         */
        minSlot: number,
        /**
         * Maximal slot from which the events should be returned (inclusive)
         *
         * @example 46154860
         */
        maxSlot: number
    },
    policyIds: PolicyId[] | undefined
};

export type MintBurnSingleResponse = {
    /**
     * Slot at which the transaction happened
     *
     * @example 512345
     */
    actionSlot: number,

    /**
     * Transaction id of related mint / burn event
     *
     * @pattern [0-9a-fA-F]{64}
     * @example "28eb069e3e8c13831d431e3b2e35f58525493ab2d77fde83184993e4aa7a0eda"
     */
    actionTxId: string,

    /**
     * Block id of related mint / burn event
     *
     * @pattern [0-9a-fA-F]{64}
     * @example "4e90f1d14ad742a1c0e094a89ad180b896068f93fc3969614b1c53bac547b374"
     */
    actionBlockId: string,

    /**
     * Transaction metadata of related mint / burn event
     */
    metadata: string | null,

    /**
     * Assets changed in a particular transaction
     *
     * @example { "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f": { "42657272794e617679": "1" }}
     */
    assets: { [policyId: string]: { [assetName: string]: Amount } };
};

export type MintBurnHistoryResponse = MintBurnSingleResponse[]