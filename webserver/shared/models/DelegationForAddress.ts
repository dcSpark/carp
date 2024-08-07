import { Address } from "./Address";

export type DelegationForAddressRequest = {
  address: Address;
  until: { absoluteSlot: number }
};

export type DelegationForAddressResponse = {
    pool: string | null;
    txId: string | null;
};

export type DrepDelegationForAddressResponse = {
    drep: string | null;
    txId: string | null;
};