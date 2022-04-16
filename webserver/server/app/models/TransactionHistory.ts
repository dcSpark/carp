import type { Address } from './Address';
import type Zapatos from 'zapatos/schema';
import { expectType } from 'tsd';

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
  num: number;
  hash: string;
  /** timestamp with timezone */
  // time: string;
  epoch: number;
  slot: number;
  /** Era of block this transaction was submitted in */
  era: number;
  /** index of tx in block */
  tx_ordinal: number;
  is_valid: boolean;
};
export type TransactionInfo = {
  block: null | BlockInfo;
  /** cbor-encoded transaction */
  transaction: string;
};
export type TransactionHistoryResponse = {
  transactions: TransactionInfo[];
};

// tsoa can't support looking up Zapatos types, so instead we just make sure the types match
expectType<Equals<BlockInfo['era'], Zapatos.Block.Selectable['era']>>(true);
