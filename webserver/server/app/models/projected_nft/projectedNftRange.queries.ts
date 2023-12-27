/** Types generated for queries found in "app/models/projected_nft/projectedNftRange.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlProjectedNftRange' parameters type */
export interface ISqlProjectedNftRangeParams {
  limit: string;
  max_slot: number;
  min_slot: number;
}

/** 'SqlProjectedNftRange' return type */
export interface ISqlProjectedNftRangeResult {
  action_output_index: number | null;
  action_slot: number;
  action_tx_id: string | null;
  amount: string;
  asset_name: string;
  for_how_long: string | null;
  owner_address: string | null;
  plutus_datum: string | null;
  policy_id: string;
  previous_tx_hash: string | null;
  previous_tx_output_index: string | null;
  status: string | null;
}

/** 'SqlProjectedNftRange' query type */
export interface ISqlProjectedNftRangeQuery {
  params: ISqlProjectedNftRangeParams;
  result: ISqlProjectedNftRangeResult;
}

const sqlProjectedNftRangeIR: any = {"usedParamSet":{"min_slot":true,"max_slot":true,"limit":true},"params":[{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":1219,"b":1228},{"a":1600,"b":1609}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":1254,"b":1263},{"a":1643,"b":1652}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1668,"b":1674}]}],"statement":"SELECT\n    encode(\"ProjectedNFT\".owner_address, 'hex') as owner_address,\n\n    encode(\"ProjectedNFT\".previous_utxo_tx_hash, 'hex') as previous_tx_hash,\n    \"ProjectedNFT\".previous_utxo_tx_output_index as previous_tx_output_index,\n\n    CASE\n        WHEN \"TransactionOutput\".output_index = NULL THEN NULL\n        ELSE \"TransactionOutput\".output_index\n        END AS action_output_index,\n\n    encode(\"Transaction\".hash, 'hex') as action_tx_id,\n\n    \"ProjectedNFT\".policy_id as policy_id,\n    \"ProjectedNFT\".asset_name as asset_name,\n    \"ProjectedNFT\".amount as amount,\n\n    CASE\n        WHEN \"ProjectedNFT\".operation = 0 THEN 'Lock'\n        WHEN \"ProjectedNFT\".operation = 1 THEN 'Unlocking'\n        WHEN \"ProjectedNFT\".operation = 2 THEN 'Claim'\n        ELSE 'Invalid'\n        END AS status,\n\n    encode(\"ProjectedNFT\".plutus_datum, 'hex') as plutus_datum,\n    \"ProjectedNFT\".for_how_long as for_how_long,\n\n    \"Block\".slot as action_slot\nFROM \"ProjectedNFT\"\n         LEFT JOIN \"TransactionOutput\" ON \"TransactionOutput\".id = \"ProjectedNFT\".hololocker_utxo_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"ProjectedNFT\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n        \"Block\".slot > :min_slot!\n    AND \"Block\".slot <= :max_slot!\n    AND \"Block\".height <= (\n        SELECT MAX(\"Heights\".height) FROM\n        (SELECT \"ProjectedNFT\".id as id, \"Block\".height as height FROM \"ProjectedNFT\"\n            JOIN \"Transaction\" ON \"Transaction\".id = \"ProjectedNFT\".tx_id\n            JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n        WHERE\n            \"Block\".slot > :min_slot!\n            AND \"Block\".slot <= :max_slot!\n        LIMIT :limit!) AS \"Heights\"\n    )\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *     encode("ProjectedNFT".owner_address, 'hex') as owner_address,
 * 
 *     encode("ProjectedNFT".previous_utxo_tx_hash, 'hex') as previous_tx_hash,
 *     "ProjectedNFT".previous_utxo_tx_output_index as previous_tx_output_index,
 * 
 *     CASE
 *         WHEN "TransactionOutput".output_index = NULL THEN NULL
 *         ELSE "TransactionOutput".output_index
 *         END AS action_output_index,
 * 
 *     encode("Transaction".hash, 'hex') as action_tx_id,
 * 
 *     "ProjectedNFT".policy_id as policy_id,
 *     "ProjectedNFT".asset_name as asset_name,
 *     "ProjectedNFT".amount as amount,
 * 
 *     CASE
 *         WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
 *         WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
 *         WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
 *         ELSE 'Invalid'
 *         END AS status,
 * 
 *     encode("ProjectedNFT".plutus_datum, 'hex') as plutus_datum,
 *     "ProjectedNFT".for_how_long as for_how_long,
 * 
 *     "Block".slot as action_slot
 * FROM "ProjectedNFT"
 *          LEFT JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".hololocker_utxo_id
 *          JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *         "Block".slot > :min_slot!
 *     AND "Block".slot <= :max_slot!
 *     AND "Block".height <= (
 *         SELECT MAX("Heights".height) FROM
 *         (SELECT "ProjectedNFT".id as id, "Block".height as height FROM "ProjectedNFT"
 *             JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
 *             JOIN "Block" ON "Transaction".block_id = "Block".id
 *         WHERE
 *             "Block".slot > :min_slot!
 *             AND "Block".slot <= :max_slot!
 *         LIMIT :limit!) AS "Heights"
 *     )
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * ```
 */
export const sqlProjectedNftRange = new PreparedQuery<ISqlProjectedNftRangeParams,ISqlProjectedNftRangeResult>(sqlProjectedNftRangeIR);


