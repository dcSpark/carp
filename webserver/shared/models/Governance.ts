import { CredentialHex } from "./Address";
import { AfterBlockPagination, UntilBlockPagination } from "./common";

export type GovernanceVotesForCredentialRequest = {
  credential: CredentialHex;
  limit?: number | undefined;
} & UntilBlockPagination &
  AfterBlockPagination;

export type GovernanceVotesForCredentialResponse = {
  votes: { govActionId: string; vote: string }[];
  txId: string;
  block: string;
}[];

export type GovernanceCredentialDidVoteRequest = {
  credential: CredentialHex;
  actionIds: string[];
} & UntilBlockPagination;

export type GovernanceCredentialDidVoteResponse = {
  actionId: string;
  txId: string;
  payload: string;
}[];
