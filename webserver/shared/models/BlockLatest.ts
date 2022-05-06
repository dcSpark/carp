export type BlockLatestRequest = {
  /**
   * Note: an offset of -1 is treated the same as an offset of +1
   *
   * It's usually best to avoid pagination on the latest block.
   *
   * In Cardano, small rollbacks of 1~2 block are very frequent and expected (read Ouroboros for why)
   * That means that using this block for pagination will often lead to your pagination being invalidated by a rollback
   * To avoid this, you can pass an `offset` from the tip for more stable pagination
   */
  offset: number;
};

export type BlockLatestResponse = {
  block: {
    era: number;
    hash: string;
    height: number;
    epoch: number;
    slot: number;
  };
};
