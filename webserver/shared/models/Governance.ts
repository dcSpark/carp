import { Address } from "./Address";
import { AfterBlockPagination, UntilBlockPagination } from "./common";

export type GovernanceVotesForAddressRequest = {
  address: Address;
  limit?: number | undefined;
} & UntilBlockPagination & AfterBlockPagination;

export type GovernanceVotesForAddressResponse = {
  votes: { govActionId: string; vote: string }[];
  txId: string;
  block: string;
}[];
