import { Address } from "./Address";
import { Pool } from "./Pool";

export type DelegationForPoolRequest = {
  pools: Pool[];
  range: { minSlot: number, maxSlot: number }
};

export type DelegationForPoolResponse = {
    credential: Address;
    isDelegation: boolean,
    txId: string | null;
}[];