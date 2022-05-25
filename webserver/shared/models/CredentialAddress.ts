import type { Credential, Bech32FullAddress } from "./Address";
import type { UntilBlockPagination, PageInfo } from "./common";

export type CredentialAddressRequest = {
  credentials: Credential[];
  after?: Bech32FullAddress;
} & UntilBlockPagination;

export type CredentialAddressResponse = {
  addresses: Bech32FullAddress[];
} & PageInfo;
