/** Types generated for queries found in "app/models/delegation/delegationForAddress.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlStakeDelegation' parameters type */
export interface ISqlStakeDelegationParams {
  credential: Buffer;
  slot: number;
}

/** 'SqlStakeDelegation' return type */
export interface ISqlStakeDelegationResult {
  pool: string | null;
  tx_id: string | null;
}

/** 'SqlStakeDelegation' query type */
export interface ISqlStakeDelegationQuery {
  params: ISqlStakeDelegationParams;
  result: ISqlStakeDelegationResult;
}

const sqlStakeDelegationIR: any = {"usedParamSet":{"credential":true,"slot":true},"params":[{"name":"credential","required":true,"transform":{"type":"scalar"},"locs":[{"a":371,"b":382}]},{"name":"slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":405,"b":410}]}],"statement":"SELECT encode(pool_credential, 'hex') as pool, encode(\"Transaction\".hash, 'hex') as tx_id\nFROM \"StakeDelegationCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \n\t\"StakeCredential\".credential = :credential! AND\n\t\"Block\".slot <= :slot!\nORDER BY (\"Block\".height, \"Transaction\".tx_index) DESC\nLIMIT 1"};

/**
 * Query generated from SQL:
 * ```
 * SELECT encode(pool_credential, 'hex') as pool, encode("Transaction".hash, 'hex') as tx_id
 * FROM "StakeDelegationCredentialRelation"
 * JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
 * JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
 * JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE 
 * 	"StakeCredential".credential = :credential! AND
 * 	"Block".slot <= :slot!
 * ORDER BY ("Block".height, "Transaction".tx_index) DESC
 * LIMIT 1
 * ```
 */
export const sqlStakeDelegation = new PreparedQuery<ISqlStakeDelegationParams,ISqlStakeDelegationResult>(sqlStakeDelegationIR);


