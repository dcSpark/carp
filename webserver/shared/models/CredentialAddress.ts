import type { Credential, Bech32FullAddress } from "./Address";
import type { Pagination, PageInfo, BlockTxPair } from "./common";

export type CredentialAddressRequest = {
  credentials: Credential[];
} & Pagination;

export type CredentialAddressResponse = {
  addresses: Bech32FullAddress[];
} & PageInfo<BlockTxPair & { address: Bech32FullAddress }>;
