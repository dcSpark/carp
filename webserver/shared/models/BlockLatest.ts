export type BlockLatestRequest = {
  /**
   * Note: an offset of -1 is treated the same as an offset of +1
   *
   * It's usually best to avoid pagination on the latest block as in Cardano, small rollbacks of 1~2 block are very frequent and expected (read Ouroboros for why)
   * That means that using this block for pagination will often lead to your pagination being invalidated by a rollback
   * To avoid this, you can pass an `offset` from the tip for more stable pagination
   */
  offset: number;
};

export type BlockSubset = {
  /**
   * @example 1
   */
  era: number;
  /**
   * @pattern [0-9a-fA-F]{64}
   * @example "cf8c63a909d91776e27f7d05457e823a9dba606a7ab499ac435e7904ee70d7c8"
   */
  hash: string;
  /**
   * @example 4512067
   */
  height: number;
  /**
   * @example 209
   */
  epoch: number;
  /**
   * @example 4924800
   */
  slot: number;
};
export type BlockLatestResponse = {
  block: BlockSubset;
};
