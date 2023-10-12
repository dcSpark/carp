/** Types generated for queries found in "app/models/delegation/delegationsForPool.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlStakeDelegationByPool' parameters type */
export interface ISqlStakeDelegationByPoolParams {
  max_slot: number;
  min_slot: number;
  pools: readonly (Buffer)[];
}

/** 'SqlStakeDelegationByPool' return type */
export interface ISqlStakeDelegationByPoolResult {
  credential: string | null;
  is_delegation: boolean | null;
  tx_id: string | null;
}

/** 'SqlStakeDelegationByPool' query type */
export interface ISqlStakeDelegationByPoolQuery {
  params: ISqlStakeDelegationByPoolParams;
  result: ISqlStakeDelegationByPoolResult;
}

const sqlStakeDelegationByPoolIR: any = {"usedParamSet":{"pools":true,"min_slot":true,"max_slot":true},"params":[{"name":"pools","required":true,"transform":{"type":"array_spread"},"locs":[{"a":160,"b":166},{"a":505,"b":511},{"a":572,"b":578}]},{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":603,"b":612}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":635,"b":644}]}],"statement":"SELECT \n\tencode(credential, 'hex') as credential,\n\tencode(\"Transaction\".hash, 'hex') as tx_id,\n\tCOALESCE(\"StakeDelegationCredentialRelation\".pool_credential IN :pools!, false) as is_delegation\nFROM \"StakeDelegationCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \n    (\n\t\t\"StakeDelegationCredentialRelation\".pool_credential IN :pools! OR\n\t \t\"StakeDelegationCredentialRelation\".previous_pool IN :pools!\n\t) AND\n\t\"Block\".slot > :min_slot! AND\n\t\"Block\".slot <= :max_slot!\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT 
 * 	encode(credential, 'hex') as credential,
 * 	encode("Transaction".hash, 'hex') as tx_id,
 * 	COALESCE("StakeDelegationCredentialRelation".pool_credential IN :pools!, false) as is_delegation
 * FROM "StakeDelegationCredentialRelation"
 * JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
 * JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
 * JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE 
 *     (
 * 		"StakeDelegationCredentialRelation".pool_credential IN :pools! OR
 * 	 	"StakeDelegationCredentialRelation".previous_pool IN :pools!
 * 	) AND
 * 	"Block".slot > :min_slot! AND
 * 	"Block".slot <= :max_slot!
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * ```
 */
export const sqlStakeDelegationByPool = new PreparedQuery<ISqlStakeDelegationByPoolParams,ISqlStakeDelegationByPoolResult>(sqlStakeDelegationByPoolIR);


