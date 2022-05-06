import type { Address } from "./Address";
import type { Pagination, RelationFilter } from "./common";

export type TransactionHistoryRequest = {
  addresses: Address[];
  /** Defaults to `RelationFilterType.NO_FILTER` */
  relationFilter?: RelationFilter;
  /** Defaults to `ADDRESS_RESPONSE_LIMIT` */
  limit?: number;
} & Pagination;

export type BlockInfo = {
  height: number;
  hash: string;
  /** timestamp with timezone */
  // time: string;
  epoch: number;
  slot: number;
  /** Era of block this transaction was submitted in */
  era: number;

  // note: the following information, in a sense, belongs to the tx
  // but we put it in the block section because we can't know it
  // until the information shows up inside a block

  /** index of tx in block */
  tx_ordinal: number;
  is_valid: boolean;
};
export type TransactionInfo = {
  /**
   * Strictly speaking, you can calculate this by hashing the payload
   * It's just provided for convenience
   */
  hash: string;
  /** cbor-encoded transaction */
  payload: string;
};

export type TxAndBlockInfo = {
  block: BlockInfo;
  transaction: TransactionInfo;
};
export type TransactionHistoryResponse = {
  transactions: TxAndBlockInfo[];
};
