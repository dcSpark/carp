import type { Credential, Bech32FullAddress } from "./Address";
import type {
  UntilBlockPagination,
  OffsetPaginationRequest,
  OffsetPaginationResponse,
} from "./common";

export type CredentialAddressRequest = {
  credentials: Credential[];
} & OffsetPaginationRequest &
  UntilBlockPagination;

export type CredentialAddressResponse = {
  addresses: Bech32FullAddress[];
} & OffsetPaginationResponse;
