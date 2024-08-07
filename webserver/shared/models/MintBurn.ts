import { PolicyId, AssetName } from "./PolicyIdAssetMap";
import { Amount, Pagination, SlotLimits } from "./common";

export type MintBurnHistoryRequest = {
  policyIds: PolicyId[] | undefined;

  /** This limits the transactions in the result to this range of slots.
   * Everything else is filtered out */
  slotLimits?: SlotLimits;

  limit?: number;
} & Pagination;

export type MintBurnSingleResponse = {
  /**
   * Assets changed in a particular transaction
   *
   * @example { "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f":
   *    {
   *      "42657272794e617679": "1",
   *      "42657272794e617680": "-2"
   *    }
   * }
   */
  assets: { [policyId: string]: { [assetName: string]: Amount } };

  /**
   * Input parts indexed by address. This mapping will only contain assets that
   * were minted or burned in this transaction.
   *
   * @example {
   *  "8200581c4b2e7295ac876cfdf70e82c1cb8df3ee5cb23d93949e3322230fc447":
   *    [ { "policyId": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f", "assetName": "42657272794e617680", "amount": "2" } ]
   * }
   */
  inputAddresses: { [address: string]: {policyId: PolicyId, assetName: AssetName, amount: Amount}[]}

  /**
   * Output parts indexed by address. This mapping will only contain assets that
   * were minted or burned in this transaction.
   *
   * @example {
   *  "8200581c4b2e7295ac876cfdf70e82c1cb8df3ee5cb23d93949e3322230fc447":
   *    [ { "policyId": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f", "assetName": "42657272794e617679", "amount": "1" } ]
   * }
   */
  outputAddresses: { [address: string]: {policyId: PolicyId, assetName: AssetName, amount: Amount}[]}

  /**
   * Slot at which the transaction happened
   *
   * @example 512345
   */
  actionSlot: number;

  /**
   * Transaction metadata of related mint / burn event
   */
  metadata: string | null;

  /**
   * Transaction id of related mint / burn event
   *
   * @pattern [0-9a-fA-F]{64}
   * @example "28eb069e3e8c13831d431e3b2e35f58525493ab2d77fde83184993e4aa7a0eda"
   */
  txId: string;
  /**
   * Block id of related mint / burn event
   *
   * @pattern [0-9a-fA-F]{64}
   * @example "4e90f1d14ad742a1c0e094a89ad180b896068f93fc3969614b1c53bac547b374"
   */
  block: string;
};

export type MintBurnHistoryResponse = MintBurnSingleResponse[];
