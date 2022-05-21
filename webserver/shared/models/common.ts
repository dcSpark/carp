/**
 * Filter which uses of the address are considered relevant for the query.
 *
 * This is a bitmask, so you can combine multiple options
 * ex: `RelationFilterType.Input & RelationFilterType.Output`
 *
 * Note: relations only apply to credentials and not to full bech32 addresses
 */
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

export type Pagination = {
  /**
   * Omitting "after" means you query starting from the genesis block.
   *
   * Note: the reason you have to specify both a tx hash AND a block hash in the "after" for pagination
   * is because this is the only way to make sure your pagination doesn't get affected by rollbacks.
   * ex: a rollback could cause a tx to be removed from one block and appear in a totally different block.
   * Specifying the block hash as well allows making sure you're paginating on the right tx in the right block.
   */
  after?: {
    /** block hash */
    block: string;
    /** tx hash */
    tx: string;
  };
  /** block hash - inclusive */
  untilBlock: string;
};

export type UtxoPointer = {
  txHash: string;
  index: number;
};
