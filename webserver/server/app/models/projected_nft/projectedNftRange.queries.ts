/** Types generated for queries found in "app/models/projected_nft/projectedNftRange.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'SqlProjectedNftRange' parameters type */
export interface ISqlProjectedNftRangeParams {
  max_slot: number;
  min_slot: number;
}

/** 'SqlProjectedNftRange' return type */
export interface ISqlProjectedNftRangeResult {
  amount: string;
  asset: string;
  output_index: number;
  plutus_datum: string | null;
  slot: number;
  status: string | null;
  tx_id: string | null;
}

/** 'SqlProjectedNftRange' query type */
export interface ISqlProjectedNftRangeQuery {
  params: ISqlProjectedNftRangeParams;
  result: ISqlProjectedNftRangeResult;
}

const sqlProjectedNftRangeIR: any = {"usedParamSet":{"min_slot":true,"max_slot":true},"params":[{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":756,"b":765}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":791,"b":800}]}],"statement":"SELECT\n    CASE\n        WHEN \"ProjectedNFT\".operation = 0 THEN 'Lock'\n        WHEN \"ProjectedNFT\".operation = 1 THEN 'Unlocking'\n        WHEN \"ProjectedNFT\".operation = 2 THEN 'Claim'\n        ELSE 'Invalid'\n        END AS status,\n    \"ProjectedNFT\".asset as asset,\n    \"ProjectedNFT\".amount as amount,\n    encode(\"Transaction\".hash, 'hex') as tx_id,\n    \"TransactionOutput\".output_index as output_index,\n    encode(\"ProjectedNFT\".plutus_datum, 'hex') as plutus_datum,\n    \"Block\".slot\nFROM \"ProjectedNFT\"\n         LEFT JOIN \"TransactionOutput\" ON \"TransactionOutput\".id = \"ProjectedNFT\".utxo_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"ProjectedNFT\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n        \"Block\".slot > :min_slot!\n    AND \"Block\".slot <= :max_slot!\nORDER BY (\"Block\".height, \"Transaction\".tx_index, \"TransactionOutput\".output_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *     CASE
 *         WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
 *         WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
 *         WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
 *         ELSE 'Invalid'
 *         END AS status,
 *     "ProjectedNFT".asset as asset,
 *     "ProjectedNFT".amount as amount,
 *     encode("Transaction".hash, 'hex') as tx_id,
 *     "TransactionOutput".output_index as output_index,
 *     encode("ProjectedNFT".plutus_datum, 'hex') as plutus_datum,
 *     "Block".slot
 * FROM "ProjectedNFT"
 *          LEFT JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".utxo_id
 *          JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *         "Block".slot > :min_slot!
 *     AND "Block".slot <= :max_slot!
 * ORDER BY ("Block".height, "Transaction".tx_index, "TransactionOutput".output_index) ASC
 * ```
 */
export const sqlProjectedNftRange = new PreparedQuery<ISqlProjectedNftRangeParams,ISqlProjectedNftRangeResult>(sqlProjectedNftRangeIR);


