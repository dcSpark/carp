/** Types generated for queries found in "app/models/delegation/delegationsForPool.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlStakeDelegationByPool' parameters type */
export interface ISqlStakeDelegationByPoolParams {
  limit: string;
  max_slot: number;
  min_slot: number;
  pools: readonly (Buffer)[];
}

/** 'SqlStakeDelegationByPool' return type */
export interface ISqlStakeDelegationByPoolResult {
  credential: string | null;
  pool: string | null;
  slot: number;
  tx_id: string | null;
}

/** 'SqlStakeDelegationByPool' query type */
export interface ISqlStakeDelegationByPoolQuery {
  params: ISqlStakeDelegationByPoolParams;
  result: ISqlStakeDelegationByPoolResult;
}

const sqlStakeDelegationByPoolIR: any = {"usedParamSet":{"pools":true,"min_slot":true,"max_slot":true,"limit":true},"params":[{"name":"pools","required":true,"transform":{"type":"array_spread"},"locs":[{"a":175,"b":181},{"a":589,"b":595},{"a":656,"b":662},{"a":1218,"b":1224},{"a":1298,"b":1304}]},{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":687,"b":696},{"a":1351,"b":1360}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":719,"b":728},{"a":1394,"b":1403}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1419,"b":1425}]}],"statement":"SELECT\n\tencode(credential, 'hex') as credential,\n\tencode(\"Transaction\".hash, 'hex') as tx_id,\n\t\"Block\".slot,\n\tCASE WHEN \"StakeDelegationCredentialRelation\".pool_credential IN :pools! THEN encode(\"StakeDelegationCredentialRelation\".pool_credential, 'hex') ELSE NULL END AS pool\nFROM \"StakeDelegationCredentialRelation\"\nJOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\nJOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\nJOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE \n    (\n\t\t\"StakeDelegationCredentialRelation\".pool_credential IN :pools! OR\n\t \t\"StakeDelegationCredentialRelation\".previous_pool IN :pools!\n\t) AND\n\t\"Block\".slot > :min_slot! AND\n\t\"Block\".slot <= :max_slot!\n    AND \"Block\".height <= (\n        SELECT MAX(\"Heights\".height) FROM\n        (SELECT \"Block\".height as height FROM \"StakeDelegationCredentialRelation\"\n            JOIN \"StakeCredential\" ON stake_credential = \"StakeCredential\".id\n            JOIN \"Transaction\" ON \"Transaction\".id = \"StakeDelegationCredentialRelation\".tx_id\n            JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n        WHERE\n            (\n                \"StakeDelegationCredentialRelation\".pool_credential IN :pools! OR\n                \"StakeDelegationCredentialRelation\".previous_pool IN :pools!\n            ) AND\n            \"Block\".slot > :min_slot!\n            AND \"Block\".slot <= :max_slot!\n        LIMIT :limit!) AS \"Heights\"\n    )\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 * 	encode(credential, 'hex') as credential,
 * 	encode("Transaction".hash, 'hex') as tx_id,
 * 	"Block".slot,
 * 	CASE WHEN "StakeDelegationCredentialRelation".pool_credential IN :pools! THEN encode("StakeDelegationCredentialRelation".pool_credential, 'hex') ELSE NULL END AS pool
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
 *     AND "Block".height <= (
 *         SELECT MAX("Heights".height) FROM
 *         (SELECT "Block".height as height FROM "StakeDelegationCredentialRelation"
 *             JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
 *             JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
 *             JOIN "Block" ON "Transaction".block_id = "Block".id
 *         WHERE
 *             (
 *                 "StakeDelegationCredentialRelation".pool_credential IN :pools! OR
 *                 "StakeDelegationCredentialRelation".previous_pool IN :pools!
 *             ) AND
 *             "Block".slot > :min_slot!
 *             AND "Block".slot <= :max_slot!
 *         LIMIT :limit!) AS "Heights"
 *     )
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * ```
 */
export const sqlStakeDelegationByPool = new PreparedQuery<ISqlStakeDelegationByPoolParams,ISqlStakeDelegationByPoolResult>(sqlStakeDelegationByPoolIR);


