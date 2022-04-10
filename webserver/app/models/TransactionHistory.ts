import type { Block } from '@prisma/client';
import type { Address } from './Address';

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

export type TransactionHistoryResponse = {
  transactions: {
    block: null | {
      num: string;
      hash: string;
      /** timestamp with timezone */
      time: string;
      epoch: number;
      slot: number;
      /** Era of block this transaction was submitted in */
      era: Block['era'];
      /** index of tx in block */
      tx_ordinal: number;
      is_valid: boolean;
    };
    /** cbor-encoded transaction */
    transaction: string;
  }[];
};
