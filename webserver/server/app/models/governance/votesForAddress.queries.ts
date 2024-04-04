/** Types generated for queries found in "app/models/governance/votesForAddress.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'VotesForAddress' parameters type */
export interface IVotesForAddressParams {
  before_tx_id?: NumberOrString | null | void;
  limit: NumberOrString;
  until_tx_id: NumberOrString;
  voter: Buffer;
}

/** 'VotesForAddress' return type */
export interface IVotesForAddressResult {
  block: string;
  txId: string;
  votes: Json;
}

/** 'VotesForAddress' query type */
export interface IVotesForAddressQuery {
  params: IVotesForAddressParams;
  result: IVotesForAddressResult;
}

const votesForAddressIR: any = {"usedParamSet":{"before_tx_id":true,"until_tx_id":true,"voter":true,"limit":true},"params":[{"name":"before_tx_id","required":false,"transform":{"type":"scalar"},"locs":[{"a":399,"b":411}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":427,"b":439}]},{"name":"voter","required":true,"transform":{"type":"scalar"},"locs":[{"a":457,"b":463}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":506,"b":512}]}],"statement":"SELECT \n    json_agg(\n        json_build_object(\n            'govActionId', encode(gov_action_id, 'hex'),\n            'vote', encode(vote, 'hex')\n        )\n    ) as \"votes!\", \n    encode(tx.hash, 'hex') as \"txId!\",\n    MIN(encode(\"Block\".hash, 'hex')) as \"block!\"\nFROM  \"GovernanceVote\"\nJOIN \"Transaction\" tx ON tx.id = \"GovernanceVote\".tx_id\nJOIN \"Block\" ON \"Block\".id = tx.block_id\nWHERE\n\ttx.id < :before_tx_id AND\n\ttx.id <= :until_tx_id! AND\n    voter = :voter!\nGROUP BY tx.id\nORDER BY tx.id DESC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT 
 *     json_agg(
 *         json_build_object(
 *             'govActionId', encode(gov_action_id, 'hex'),
 *             'vote', encode(vote, 'hex')
 *         )
 *     ) as "votes!", 
 *     encode(tx.hash, 'hex') as "txId!",
 *     MIN(encode("Block".hash, 'hex')) as "block!"
 * FROM  "GovernanceVote"
 * JOIN "Transaction" tx ON tx.id = "GovernanceVote".tx_id
 * JOIN "Block" ON "Block".id = tx.block_id
 * WHERE
 * 	tx.id < :before_tx_id AND
 * 	tx.id <= :until_tx_id! AND
 *     voter = :voter!
 * GROUP BY tx.id
 * ORDER BY tx.id DESC
 * LIMIT :limit!
 * ```
 */
export const votesForAddress = new PreparedQuery<IVotesForAddressParams,IVotesForAddressResult>(votesForAddressIR);


/** 'DidVote' parameters type */
export interface IDidVoteParams {
  gov_action_ids: readonly (Buffer)[];
  until_tx_id: NumberOrString;
  voter: Buffer;
}

/** 'DidVote' return type */
export interface IDidVoteResult {
  govActionId: Buffer;
  txId: string;
  vote: Buffer;
}

/** 'DidVote' query type */
export interface IDidVoteQuery {
  params: IDidVoteParams;
  result: IDidVoteResult;
}

const didVoteIR: any = {"usedParamSet":{"until_tx_id":true,"voter":true,"gov_action_ids":true},"params":[{"name":"gov_action_ids","required":true,"transform":{"type":"array_spread"},"locs":[{"a":293,"b":308}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":230,"b":242}]},{"name":"voter","required":true,"transform":{"type":"scalar"},"locs":[{"a":260,"b":266}]}],"statement":"SELECT gov_action_id as \"govActionId!\",\n       vote as \"vote!\",\n       encode(\"Transaction\".hash, 'hex') as \"txId!\"\nFROM  \"GovernanceVote\"\nJOIN \"Transaction\" ON \"GovernanceVote\".tx_id = \"Transaction\".id\nWHERE\n\t\"Transaction\".id <= :until_tx_id! AND\n    voter = :voter! AND\n    gov_action_id IN :gov_action_ids!\nORDER BY \"Transaction\".id"};

/**
 * Query generated from SQL:
 * ```
 * SELECT gov_action_id as "govActionId!",
 *        vote as "vote!",
 *        encode("Transaction".hash, 'hex') as "txId!"
 * FROM  "GovernanceVote"
 * JOIN "Transaction" ON "GovernanceVote".tx_id = "Transaction".id
 * WHERE
 * 	"Transaction".id <= :until_tx_id! AND
 *     voter = :voter! AND
 *     gov_action_id IN :gov_action_ids!
 * ORDER BY "Transaction".id
 * ```
 */
export const didVote = new PreparedQuery<IDidVoteParams,IDidVoteResult>(didVoteIR);


