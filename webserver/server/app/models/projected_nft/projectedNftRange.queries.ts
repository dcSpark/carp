/** Types generated for queries found in "app/models/projected_nft/projectedNftRange.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'SqlProjectedNftRange' parameters type */
export interface ISqlProjectedNftRangeParams {
  after_tx_id: NumberOrString;
  limit: NumberOrString;
  until_tx_id: NumberOrString;
}

/** 'SqlProjectedNftRange' return type */
export interface ISqlProjectedNftRangeResult {
  block: string;
  payload: Json;
  tx_id: string;
}

/** 'SqlProjectedNftRange' query type */
export interface ISqlProjectedNftRangeQuery {
  params: ISqlProjectedNftRangeParams;
  result: ISqlProjectedNftRangeResult;
}

const sqlProjectedNftRangeIR: any = {"usedParamSet":{"after_tx_id":true,"until_tx_id":true,"limit":true},"params":[{"name":"after_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1369,"b":1381}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1408,"b":1420}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1522,"b":1528}]}],"statement":"SELECT\n    json_agg(json_build_object(\n        'ownerAddress', encode(\"ProjectedNFT\".owner_address, 'hex'),\n        'previousUtxoTxHash', encode(\"ProjectedNFT\".previous_utxo_tx_hash, 'hex'),\n        'previousTxOutputIndex', \"ProjectedNFT\".previous_utxo_tx_output_index,\n        'actionOutputIndex', CASE\n            WHEN \"TransactionOutput\".output_index = NULL THEN NULL\n            ELSE \"TransactionOutput\".output_index\n            END,\n        'policyId', \"ProjectedNFT\".policy_id,\n        'assetName', \"ProjectedNFT\".asset_name,\n        'amount', \"ProjectedNFT\".amount,\n        'status', CASE\n            WHEN \"ProjectedNFT\".operation = 0 THEN 'Lock'\n            WHEN \"ProjectedNFT\".operation = 1 THEN 'Unlocking'\n            WHEN \"ProjectedNFT\".operation = 2 THEN 'Claim'\n            ELSE 'Invalid'\n            END,\n        'plutusDatum', encode(\"ProjectedNFT\".plutus_datum, 'hex'),\n        'forHowLong', \"ProjectedNFT\".for_how_long,\n        'actionSlot', \"Block\".slot\n    )) as \"payload!\",\n    encode(\"Block\".hash, 'hex') as \"block!\",\n    encode(\"Transaction\".hash, 'hex') as \"tx_id!\"\nFROM \"ProjectedNFT\"\n         LEFT JOIN \"TransactionOutput\" ON \"TransactionOutput\".id = \"ProjectedNFT\".hololocker_utxo_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"ProjectedNFT\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n\t\"Transaction\".id > :after_tx_id! AND\n\t\"Transaction\".id <= :until_tx_id!\nGROUP BY (\"Block\".id, \"Transaction\".id)\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *     json_agg(json_build_object(
 *         'ownerAddress', encode("ProjectedNFT".owner_address, 'hex'),
 *         'previousUtxoTxHash', encode("ProjectedNFT".previous_utxo_tx_hash, 'hex'),
 *         'previousTxOutputIndex', "ProjectedNFT".previous_utxo_tx_output_index,
 *         'actionOutputIndex', CASE
 *             WHEN "TransactionOutput".output_index = NULL THEN NULL
 *             ELSE "TransactionOutput".output_index
 *             END,
 *         'policyId', "ProjectedNFT".policy_id,
 *         'assetName', "ProjectedNFT".asset_name,
 *         'amount', "ProjectedNFT".amount,
 *         'status', CASE
 *             WHEN "ProjectedNFT".operation = 0 THEN 'Lock'
 *             WHEN "ProjectedNFT".operation = 1 THEN 'Unlocking'
 *             WHEN "ProjectedNFT".operation = 2 THEN 'Claim'
 *             ELSE 'Invalid'
 *             END,
 *         'plutusDatum', encode("ProjectedNFT".plutus_datum, 'hex'),
 *         'forHowLong', "ProjectedNFT".for_how_long,
 *         'actionSlot', "Block".slot
 *     )) as "payload!",
 *     encode("Block".hash, 'hex') as "block!",
 *     encode("Transaction".hash, 'hex') as "tx_id!"
 * FROM "ProjectedNFT"
 *          LEFT JOIN "TransactionOutput" ON "TransactionOutput".id = "ProjectedNFT".hololocker_utxo_id
 *          JOIN "Transaction" ON "Transaction".id = "ProjectedNFT".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 * 	"Transaction".id > :after_tx_id! AND
 * 	"Transaction".id <= :until_tx_id!
 * GROUP BY ("Block".id, "Transaction".id)
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * LIMIT :limit!
 * ```
 */
export const sqlProjectedNftRange = new PreparedQuery<ISqlProjectedNftRangeParams,ISqlProjectedNftRangeResult>(sqlProjectedNftRangeIR);


