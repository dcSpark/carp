import type { Address } from "./Address";

export type RelationFilter = number;
// note: keep in sync with Rust type TxCredentialRelationValue
export enum RelationFilterType {
  FILTER_ALL = 0,
  Witness = 0b1,
  Input = 0b10,
  Output = 0b100,
  StakeDeregistration = 0b1000,
  StakeDelegation = 0b10000,
  StakeRegistration = 0b100000,
  DelegationTarget = 0b1000000,
  PoolOwner = 0b10000000,
  PoolOperator = 0b100000000,
  PoolReward = 0b1000000000,
  MirRecipient = 0b10000000000,
  Withdrawal = 0b100000000000,
  RequiredSigner = 0b1000000000000,
  InNativeScript = 0b10000000000000,
  UnusedInput = 0b100000000000000,
  UnusedInputStake = 0b1000000000000000,
  InputStake = 0b10000000000000000,
  OutputStake = 0b100000000000000000,
  NO_FILTER = 0xff,
}

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
  relationFilter?: RelationFilter;
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

export type TxAndBlockInfo = {
  block: BlockInfo;
  transaction: TransactionInfo;
};
export type TransactionHistoryResponse = {
  transactions: TxAndBlockInfo[];
};
