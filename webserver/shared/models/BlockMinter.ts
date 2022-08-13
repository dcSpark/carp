import { BlockHash } from "./common";

export type BlockMinterRequest = {
  hash: BlockHash;
};
export type BlockMinterResponse = {
  // TODO: example and pattern
  pubkey: string;
};
