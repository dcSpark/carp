import { Address } from "./Address";
import { Pool, PoolHex } from "./Pool";

export type DelegationForPoolRequest = {
  pools: Pool[];
  range: { minSlot: number, maxSlot: number }
};

export type DelegationForPoolResponse = {
    credential: Address;
    pool: PoolHex | null,
    txId: string;
    slot: number;
}[];