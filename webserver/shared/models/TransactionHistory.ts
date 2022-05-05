import type { Address } from "./Address";

export type TransactionHistoryRequest = {
  addresses: Address[];
  /** omitting "after" means you query starting from the genesis block */
  after?: {
    /** block hash */
    block: string;
    /** tx hash */
    tx: string;
  };
  /** block hash - inclusive */
  untilBlock: string;
};

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

// https://github.com/CardanoSolutions/ogmios/issues/209
export type MempoolTx = {
  hash: string;
  positionInMempool: number;
};
export type TxAndBlockInfo = {
  block: BlockInfo;
  transaction: TransactionInfo;
};
export type TransactionHistoryResponse = {
  transactions: TxAndBlockInfo[];
};
