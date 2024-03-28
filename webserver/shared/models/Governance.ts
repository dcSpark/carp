import { CredentialHex } from "./Address";
import { AfterBlockPagination, UntilBlockPagination } from "./common";

export type GovernanceVotesForAddressRequest = {
  credential: CredentialHex;
  limit?: number | undefined;
} & UntilBlockPagination & AfterBlockPagination;

export type GovernanceVotesForAddressResponse = {
  votes: { govActionId: string; vote: string }[];
  txId: string;
  block: string;
}[];
