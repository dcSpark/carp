import { AssetName, PolicyId } from "./PolicyIdAssetMap";

/**
 * Filter which uses of the address are considered relevant for the query.
 *
 * This is a bitmask, so you can combine multiple options
 * ex: `RelationFilterType.Input | RelationFilterType.Output`
 *
 * Note: relations only apply to credentials and not to full bech32 addresses
 * @pattern ([01]?[0-9]?[0-9]|2[0-4][0-9]|25[0-5])
 * @example 255
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
  UnusedOutput = 0b1000000000000000000,
  UnusedOutputStake = 0b10000000000000000000,
  ReferenceInput = 0b100000000000000000000,
  ReferenceInputStake = 0b1000000000000000000000,
  NO_FILTER = 0xff,
}

/**
* @pattern [0-9a-fA-F]*
*/
export type UnboundHex = string;

export type BlockTxPair = {
  /**
   * block hash
   * @pattern [0-9a-fA-F]{64}
   * @example "2548ad5d0d9d33d50ab43151f574474454017a733e307229fa509c4987ca9782"
   */
  block: string;
  /**
   * tx hash
   * @pattern [0-9a-fA-F]{64}
   * @example "336d520af58ff440b2f20210ddb5ef5b2c035e0ec7ec258bae4b519a87fa1696"
   */
  tx: string;
};
export type AfterBlockPagination = {
  /**
   * Omitting "after" means you query starting from the genesis block.
   *
   * Note: the reason you have to specify both a tx hash AND a block hash in the "after" for pagination
   * is because this is the only way to make sure your pagination doesn't get affected by rollbacks.
   * ex: a rollback could cause a tx to be removed from one block and appear in a totally different block.
   * Specifying the block hash as well allows making sure you're paginating on the right tx in the right block.
   */
  after?: BlockTxPair;
};
export type UntilBlockPagination = {
  /**
   * block hash - inclusive
   * @pattern [0-9a-fA-F]{64}
   * @example "cf8c63a909d91776e27f7d05457e823a9dba606a7ab499ac435e7904ee70d7c8"
   */
  untilBlock: string;
};
export type Pagination = AfterBlockPagination & UntilBlockPagination;

export type UtxoPointer = {
  /**
   * @pattern [0-9a-fA-F]{64}
   * @example "011b86557367525891331b4bb985545120efc335b606d6a1c0d5a35fb330f421"
   */
  txHash: string;
  index: number;
};

export type PageInfo = {
  pageInfo: {
    /**
     * @example false
     */
    hasNextPage: boolean;
  };
};

export enum Direction {
  Buy = 'buy',
  Sell = 'sell',
};

export enum Dex {
  WingRiders = 'WingRiders',
  SundaeSwap = 'SundaeSwap',
  MinSwap = 'MinSwap',
};

export type Asset = {
  policyId: PolicyId;
  assetName: AssetName;
} | null;

/**
 * @pattern [1-9][0-9]*
 * @example "2042352568679"
 */
export type Amount = string;

export type SlotLimits = {
  // this is exclusive
  from: number;
  // this is inclusive
  to: number;
};
