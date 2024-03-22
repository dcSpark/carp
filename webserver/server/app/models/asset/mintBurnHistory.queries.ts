/** Types generated for queries found in "app/models/asset/mintBurnHistory.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type BufferArray = (Buffer)[];

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'SqlMintBurnRange' parameters type */
export interface ISqlMintBurnRangeParams {
  after_tx_id: NumberOrString;
  limit: NumberOrString;
  until_tx_id: NumberOrString;
}

/** 'SqlMintBurnRange' return type */
export interface ISqlMintBurnRangeResult {
  action_slot: number;
  action_tx_metadata: string | null;
  block: string;
  input_payloads: BufferArray | null;
  output_payloads: BufferArray | null;
  payload: Json;
  tx: string;
}

/** 'SqlMintBurnRange' query type */
export interface ISqlMintBurnRangeQuery {
  params: ISqlMintBurnRangeParams;
  result: ISqlMintBurnRangeResult;
}

const sqlMintBurnRangeIR: any = {"usedParamSet":{"after_tx_id":true,"until_tx_id":true,"limit":true},"params":[{"name":"after_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1118,"b":1130}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1157,"b":1169}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1271,"b":1277}]}],"statement":"SELECT\n\tENCODE(\"Transaction\".HASH, 'hex') \"tx!\",\n\tENCODE(\"Block\".HASH, 'hex') AS \"block!\",\n\t\"Block\".slot AS action_slot,\n\tENCODE(\"TransactionMetadata\".payload, 'hex') as action_tx_metadata,\n\tjson_agg(json_build_object(\n        'amount', \"AssetMint\".amount::text,\n        'policyId', encode(\"NativeAsset\".policy_id, 'hex'),\n        'assetName', encode(\"NativeAsset\".asset_name, 'hex')\n\t)) as \"payload!\",\n\tarray_agg(DISTINCT \"TransactionOutput\".payload) as output_payloads,\n\tarray_agg(DISTINCT input_outputs.payload) input_payloads\nFROM \"AssetMint\"\n         LEFT JOIN \"TransactionMetadata\" ON \"TransactionMetadata\".id = \"AssetMint\".tx_id\n         JOIN \"NativeAsset\" ON \"NativeAsset\".id = \"AssetMint\".asset_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"AssetMint\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n\t\t LEFT JOIN \"TransactionOutput\" ON \"TransactionOutput\".tx_id = \"Transaction\".id\n\t\t LEFT JOIN \"TransactionInput\" ON \"TransactionInput\".tx_id = \"Transaction\".id\n\t\t LEFT JOIN \"TransactionOutput\" input_outputs ON \"TransactionInput\".utxo_id = input_outputs.id\nWHERE\n\t\"Transaction\".id > :after_tx_id! AND\n\t\"Transaction\".id <= :until_tx_id!\nGROUP BY \"Transaction\".id, \"Block\".id, \"TransactionMetadata\".id\nORDER BY \"Transaction\".id ASC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 * 	ENCODE("Transaction".HASH, 'hex') "tx!",
 * 	ENCODE("Block".HASH, 'hex') AS "block!",
 * 	"Block".slot AS action_slot,
 * 	ENCODE("TransactionMetadata".payload, 'hex') as action_tx_metadata,
 * 	json_agg(json_build_object(
 *         'amount', "AssetMint".amount::text,
 *         'policyId', encode("NativeAsset".policy_id, 'hex'),
 *         'assetName', encode("NativeAsset".asset_name, 'hex')
 * 	)) as "payload!",
 * 	array_agg(DISTINCT "TransactionOutput".payload) as output_payloads,
 * 	array_agg(DISTINCT input_outputs.payload) input_payloads
 * FROM "AssetMint"
 *          LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
 *          JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
 *          JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * 		 LEFT JOIN "TransactionOutput" ON "TransactionOutput".tx_id = "Transaction".id
 * 		 LEFT JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
 * 		 LEFT JOIN "TransactionOutput" input_outputs ON "TransactionInput".utxo_id = input_outputs.id
 * WHERE
 * 	"Transaction".id > :after_tx_id! AND
 * 	"Transaction".id <= :until_tx_id!
 * GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
 * ORDER BY "Transaction".id ASC
 * LIMIT :limit!
 * ```
 */
export const sqlMintBurnRange = new PreparedQuery<ISqlMintBurnRangeParams,ISqlMintBurnRangeResult>(sqlMintBurnRangeIR);


/** 'SqlMintBurnRangeByPolicyIds' parameters type */
export interface ISqlMintBurnRangeByPolicyIdsParams {
  after_tx_id: NumberOrString;
  limit: NumberOrString;
  policy_ids: readonly (Buffer)[];
  until_tx_id: NumberOrString;
}

/** 'SqlMintBurnRangeByPolicyIds' return type */
export interface ISqlMintBurnRangeByPolicyIdsResult {
  action_slot: number;
  action_tx_metadata: string | null;
  block: string;
  input_payloads: BufferArray | null;
  output_payloads: BufferArray | null;
  payload: Json;
  tx: string;
}

/** 'SqlMintBurnRangeByPolicyIds' query type */
export interface ISqlMintBurnRangeByPolicyIdsQuery {
  params: ISqlMintBurnRangeByPolicyIdsParams;
  result: ISqlMintBurnRangeByPolicyIdsResult;
}

const sqlMintBurnRangeByPolicyIdsIR: any = {"usedParamSet":{"after_tx_id":true,"until_tx_id":true,"policy_ids":true,"limit":true},"params":[{"name":"policy_ids","required":true,"transform":{"type":"array_spread"},"locs":[{"a":1206,"b":1217}]},{"name":"after_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1118,"b":1130}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1157,"b":1169}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1319,"b":1325}]}],"statement":"SELECT\n\tENCODE(\"Transaction\".HASH, 'hex') \"tx!\",\n\tENCODE(\"Block\".HASH, 'hex') AS \"block!\",\n\t\"Block\".slot AS action_slot,\n\tENCODE(\"TransactionMetadata\".payload, 'hex') as action_tx_metadata,\n\tjson_agg(json_build_object(\n        'amount', \"AssetMint\".amount::text,\n        'policyId', encode(\"NativeAsset\".policy_id, 'hex'),\n        'assetName', encode(\"NativeAsset\".asset_name, 'hex')\n\t)) as \"payload!\",\n\tarray_agg(DISTINCT \"TransactionOutput\".payload) as output_payloads,\n\tarray_agg(DISTINCT input_outputs.payload) input_payloads\nFROM \"AssetMint\"\n         LEFT JOIN \"TransactionMetadata\" ON \"TransactionMetadata\".id = \"AssetMint\".tx_id\n         JOIN \"NativeAsset\" ON \"NativeAsset\".id = \"AssetMint\".asset_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"AssetMint\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n\t\t LEFT JOIN \"TransactionOutput\" ON \"TransactionOutput\".tx_id = \"Transaction\".id\n\t\t LEFT JOIN \"TransactionInput\" ON \"TransactionInput\".tx_id = \"Transaction\".id\n\t\t LEFT JOIN \"TransactionOutput\" input_outputs ON \"TransactionInput\".utxo_id = input_outputs.id\nWHERE\n\t\"Transaction\".id > :after_tx_id! AND\n\t\"Transaction\".id <= :until_tx_id!\n    AND \"NativeAsset\".policy_id IN :policy_ids!\nGROUP BY \"Transaction\".id, \"Block\".id, \"TransactionMetadata\".id\nORDER BY \"Transaction\".id ASC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 * 	ENCODE("Transaction".HASH, 'hex') "tx!",
 * 	ENCODE("Block".HASH, 'hex') AS "block!",
 * 	"Block".slot AS action_slot,
 * 	ENCODE("TransactionMetadata".payload, 'hex') as action_tx_metadata,
 * 	json_agg(json_build_object(
 *         'amount', "AssetMint".amount::text,
 *         'policyId', encode("NativeAsset".policy_id, 'hex'),
 *         'assetName', encode("NativeAsset".asset_name, 'hex')
 * 	)) as "payload!",
 * 	array_agg(DISTINCT "TransactionOutput".payload) as output_payloads,
 * 	array_agg(DISTINCT input_outputs.payload) input_payloads
 * FROM "AssetMint"
 *          LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
 *          JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
 *          JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * 		 LEFT JOIN "TransactionOutput" ON "TransactionOutput".tx_id = "Transaction".id
 * 		 LEFT JOIN "TransactionInput" ON "TransactionInput".tx_id = "Transaction".id
 * 		 LEFT JOIN "TransactionOutput" input_outputs ON "TransactionInput".utxo_id = input_outputs.id
 * WHERE
 * 	"Transaction".id > :after_tx_id! AND
 * 	"Transaction".id <= :until_tx_id!
 *     AND "NativeAsset".policy_id IN :policy_ids!
 * GROUP BY "Transaction".id, "Block".id, "TransactionMetadata".id
 * ORDER BY "Transaction".id ASC
 * LIMIT :limit!
 * ```
 */
export const sqlMintBurnRangeByPolicyIds = new PreparedQuery<ISqlMintBurnRangeByPolicyIdsParams,ISqlMintBurnRangeByPolicyIdsResult>(sqlMintBurnRangeByPolicyIdsIR);


