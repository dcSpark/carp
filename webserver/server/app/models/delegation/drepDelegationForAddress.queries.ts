/** Types generated for queries found in "app/models/delegation/drepDelegationForAddress.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'SqlDrepStakeDelegationForAddress' parameters type */
export interface ISqlDrepStakeDelegationForAddressParams {
  credential: Buffer;
  slot: number;
}

/** 'SqlDrepStakeDelegationForAddress' return type */
export interface ISqlDrepStakeDelegationForAddressResult {
  drep: string;
  tx_id: string;
}

/** 'SqlDrepStakeDelegationForAddress' query type */
export interface ISqlDrepStakeDelegationForAddressQuery {
  params: ISqlDrepStakeDelegationForAddressParams;
  result: ISqlDrepStakeDelegationForAddressResult;
}

const sqlDrepStakeDelegationForAddressIR: any = {"usedParamSet":{"credential":true,"slot":true},"params":[{"name":"credential","required":true,"transform":{"type":"scalar"},"locs":[{"a":384,"b":395}]},{"name":"slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":418,"b":423}]}],"statement":"SELECT encode(drep_credential, 'hex') as \"drep!\", encode(\"Transaction\".hash, 'hex') as \"tx_id!\"\nFROM \"StakeDelegationDrepCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationDrepCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n\t\"StakeCredential\".credential = :credential! AND\n\t\"Block\".slot <= :slot!\nORDER BY \"Transaction\".id DESC\nLIMIT 1"};

/**
 * Query generated from SQL:
 * ```
 * SELECT encode(drep_credential, 'hex') as "drep!", encode("Transaction".hash, 'hex') as "tx_id!"
 * FROM "StakeDelegationDrepCredentialRelation"
 * JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
 * JOIN "Transaction" ON "Transaction".id = "StakeDelegationDrepCredentialRelation".tx_id
 * JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 * 	"StakeCredential".credential = :credential! AND
 * 	"Block".slot <= :slot!
 * ORDER BY "Transaction".id DESC
 * LIMIT 1
 * ```
 */
export const sqlDrepStakeDelegationForAddress = new PreparedQuery<ISqlDrepStakeDelegationForAddressParams,ISqlDrepStakeDelegationForAddressResult>(sqlDrepStakeDelegationForAddressIR);


