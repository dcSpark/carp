/** Types generated for queries found in "app/models/delegation/delegationsForPool.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'SqlStakeDelegationByPool' parameters type */
export interface ISqlStakeDelegationByPoolParams {
  after_tx_id: NumberOrString;
  limit: NumberOrString;
  pools: readonly (Buffer)[];
  until_tx_id: NumberOrString;
}

/** 'SqlStakeDelegationByPool' return type */
export interface ISqlStakeDelegationByPoolResult {
  block: string;
  payload: Json;
  tx_id: string;
}

/** 'SqlStakeDelegationByPool' query type */
export interface ISqlStakeDelegationByPoolQuery {
  params: ISqlStakeDelegationByPoolParams;
  result: ISqlStakeDelegationByPoolResult;
}

const sqlStakeDelegationByPoolIR: any = {"usedParamSet":{"pools":true,"after_tx_id":true,"until_tx_id":true,"limit":true},"params":[{"name":"pools","required":true,"transform":{"type":"array_spread"},"locs":[{"a":272,"b":278},{"a":708,"b":714},{"a":775,"b":781}]},{"name":"after_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":810,"b":822}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":849,"b":861}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":941,"b":947}]}],"statement":"SELECT \n\tencode(\"Transaction\".hash, 'hex') as \"tx_id!\",\n\tencode(\"Block\".hash, 'hex') as \"block!\",\n\tjson_agg(json_build_object(\n\t\t'credential', encode(credential, 'hex'),\n\t\t'slot', \"Block\".slot,\n\t\t'pool',\n\t\t\tCASE WHEN \"StakeDelegationCredentialRelation\".pool_credential IN :pools!\n\t\t\tTHEN encode(\"StakeDelegationCredentialRelation\".pool_credential, 'hex')\n\t\t\tELSE NULL\n\t\t\tEND\n\t\t)\n\t) as \"payload!\"\nFROM \"StakeDelegationCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \n    (\n\t\t\"StakeDelegationCredentialRelation\".pool_credential IN :pools! OR\n\t \t\"StakeDelegationCredentialRelation\".previous_pool IN :pools!\n\t) AND\n\t\"Transaction\".id > :after_tx_id! AND\n\t\"Transaction\".id <= :until_tx_id!\nGROUP BY (\"Block\".hash, \"Transaction\".id)\nORDER BY \"Transaction\".id ASC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT 
 * 	encode("Transaction".hash, 'hex') as "tx_id!",
 * 	encode("Block".hash, 'hex') as "block!",
 * 	json_agg(json_build_object(
 * 		'credential', encode(credential, 'hex'),
 * 		'slot', "Block".slot,
 * 		'pool',
 * 			CASE WHEN "StakeDelegationCredentialRelation".pool_credential IN :pools!
 * 			THEN encode("StakeDelegationCredentialRelation".pool_credential, 'hex')
 * 			ELSE NULL
 * 			END
 * 		)
 * 	) as "payload!"
 * FROM "StakeDelegationCredentialRelation"
 * JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
 * JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
 * JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE 
 *     (
 * 		"StakeDelegationCredentialRelation".pool_credential IN :pools! OR
 * 	 	"StakeDelegationCredentialRelation".previous_pool IN :pools!
 * 	) AND
 * 	"Transaction".id > :after_tx_id! AND
 * 	"Transaction".id <= :until_tx_id!
 * GROUP BY ("Block".hash, "Transaction".id)
 * ORDER BY "Transaction".id ASC
 * LIMIT :limit!
 * ```
 */
export const sqlStakeDelegationByPool = new PreparedQuery<ISqlStakeDelegationByPoolParams,ISqlStakeDelegationByPoolResult>(sqlStakeDelegationByPoolIR);


