import type { Address } from "./Address";
import type { BlockSubset } from "./BlockLatest";
import { AssetName, PolicyId } from "./PolicyIdAssetMap";
import type { Pagination, RelationFilter } from "./common";

export type TransactionHistoryRequest = {
  addresses: Address[];
  /** Defaults to `RelationFilterType.NO_FILTER` */
  relationFilter?: RelationFilter;
  /** Defaults to `ADDRESS_LIMIT.RESPONSE` */
  limit?: number;

  slotLimits?: SlotLimits;
} & Pagination;

export type BlockInfo = BlockSubset & {
  // note: the following information, in a sense, belongs to the tx
  // but we put it in the block section because we can't know it
  // until the information shows up inside a block

  /** index of tx in block */
  indexInBlock: number;
  isValid: boolean;
};
export type TransactionInfo = {
  /**
   * Strictly speaking, you can calculate this by hashing the payload
   * It's just provided for convenience
   * @pattern [0-9a-fA-F]{64}
   * @example "011b86557367525891331b4bb985545120efc335b606d6a1c0d5a35fb330f421"
   */
  hash: string;
  /**
   * cbor-encoded transaction
   * @pattern [0-9a-fA-F]*
   * @example "84a500818258209cb4f8c2eecccc9f1e13768046f37ef56dcb5a4dc44f58907fe4ae21d7cf621d020181825839019cb581f4337a6142e477af6e00fe41b1fc4a5944a575681b8499a3c0bd07ce733b5911eb657e7aff5d35f8b0682fe0380f7621af2bbcb2f71b0000000586321393021a0002a389031a004b418c048183028200581cbd07ce733b5911eb657e7aff5d35f8b0682fe0380f7621af2bbcb2f7581c53215c471b7ac752e3ddf8f2c4c1e6ed111857bfaa675d5e31ce8bcea1008282582073e584cda9fe483fbefb81c251e616018a2b493ef56820f0095b63adede54ff758404f13df42ef1684a3fd55255d8368c9ecbd15b55e2761a2991cc4f401a753c16d6da1da158e84b87b4de9715af7d9adc0d79a7c1f2c3097228e02b20be4616a0c82582066c606974819f457ceface78ee3c4d181a84ca9927a3cfc92ef8c0b6dd4576e8584014ae9ee9ed5eb5700b6c5ac270543671f5d4f943d4726f4614dc061174ee29db44b9e7fc58e6c98c13fad8594f2633c5ec70a9a87f5cbf130308a42edb553001f5f6"
   */
  payload: string;

  outputs: {
    asset: { policyId: PolicyId; assetName: AssetName } | null;
    amount: string;
    address: string;
  }[];

  metadata: string | null;

  inputCredentials: string[];
};

export type TxAndBlockInfo = {
  block: BlockInfo;
  transaction: TransactionInfo;
};
export type TransactionHistoryResponse = {
  transactions: TxAndBlockInfo[];
};

export type SlotLimits = {
  from: number;
  to: number;
};
