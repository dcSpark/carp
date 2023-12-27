import { Address } from "./Address";
import { Pool, PoolHex } from "./Pool";
import type { SlotPagination } from "./common";

export type DelegationForPoolRequest = {
  pools: Pool[];
  limit: number | undefined
} & SlotPagination;

export type DelegationForPoolSingleResponse = {
    credential: Address;
    pool: PoolHex | null,
    txId: string | null;
    slot: number;
};

export type DelegationForPoolResponse = {
    result: DelegationForPoolSingleResponse[],
    after: number | undefined,
};