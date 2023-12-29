/** Types generated for queries found in "app/models/delegation/delegationForAddress.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'SqlStakeDelegationForAddress' parameters type */
export interface ISqlStakeDelegationForAddressParams {
  credential: Buffer;
  slot: number;
}

/** 'SqlStakeDelegationForAddress' return type */
export interface ISqlStakeDelegationForAddressResult {
  pool: string;
  tx_id: string;
}

/** 'SqlStakeDelegationForAddress' query type */
export interface ISqlStakeDelegationForAddressQuery {
  params: ISqlStakeDelegationForAddressParams;
  result: ISqlStakeDelegationForAddressResult;
}

const sqlStakeDelegationForAddressIR: any = {"usedParamSet":{"credential":true,"slot":true},"params":[{"name":"credential","required":true,"transform":{"type":"scalar"},"locs":[{"a":377,"b":388}]},{"name":"slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":411,"b":416}]}],"statement":"SELECT encode(pool_credential, 'hex') as \"pool!\", encode(\"Transaction\".hash, 'hex') as \"tx_id!\"\nFROM \"StakeDelegationCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \n\t\"StakeCredential\".credential = :credential! AND\n\t\"Block\".slot <= :slot!\nORDER BY (\"Block\".height, \"Transaction\".tx_index) DESC\nLIMIT 1"};

/**
 * Query generated from SQL:
 * ```
 * SELECT encode(pool_credential, 'hex') as "pool!", encode("Transaction".hash, 'hex') as "tx_id!"
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
export const sqlStakeDelegationForAddress = new PreparedQuery<ISqlStakeDelegationForAddressParams,ISqlStakeDelegationForAddressResult>(sqlStakeDelegationForAddressIR);


