import type { Pagination, SlotLimits } from "./common";
import { Address } from "./Address";
import { Pool, PoolHex } from "./Pool";

export type DelegationForPoolRequest = {
  pools: Pool[];

  /** This limits the transactions in the result to this range of slots.
   * Everything else is filtered out */
  slotLimits?: SlotLimits;

  limit?: number;
} & Pagination;

export type DelegationForPoolResponse = {
  payload: {
    credential: Address;
    pool: PoolHex | null;
    slot: number;
  }[];
  txId: string;
  block: string;
}[];
