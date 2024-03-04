/** Types generated for queries found in "app/models/asset/mintBurnHistory.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'SqlMintBurnRange' parameters type */
export interface ISqlMintBurnRangeParams {
  max_slot: number;
  min_slot: number;
}

/** 'SqlMintBurnRange' return type */
export interface ISqlMintBurnRangeResult {
  action_block_id: string | null;
  action_slot: number;
  action_tx_id: string | null;
  action_tx_metadata: string | null;
  amount: string;
  asset_name: string | null;
  policy_id: string | null;
}

/** 'SqlMintBurnRange' query type */
export interface ISqlMintBurnRangeQuery {
  params: ISqlMintBurnRangeParams;
  result: ISqlMintBurnRangeResult;
}

const sqlMintBurnRangeIR: any = {"usedParamSet":{"min_slot":true,"max_slot":true},"params":[{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":793,"b":802}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":828,"b":837}]}],"statement":"SELECT\n    \"AssetMint\".amount as amount,\n    encode(\"NativeAsset\".policy_id, 'hex') as policy_id,\n    encode(\"NativeAsset\".asset_name, 'hex') as asset_name,\n    encode(\"Transaction\".hash, 'hex') as action_tx_id,\n    encode(\"Block\".hash, 'hex') as action_block_id,\n    CASE\n        WHEN \"TransactionMetadata\".payload = NULL THEN NULL\n        ELSE encode(\"TransactionMetadata\".payload, 'hex')\n        END AS action_tx_metadata,\n    \"Block\".slot as action_slot\nFROM \"AssetMint\"\n         LEFT JOIN \"TransactionMetadata\" ON \"TransactionMetadata\".id = \"AssetMint\".tx_id\n         JOIN \"NativeAsset\" ON \"NativeAsset\".id = \"AssetMint\".asset_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"AssetMint\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n        \"Block\".slot > :min_slot!\n    AND \"Block\".slot <= :max_slot!\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *     "AssetMint".amount as amount,
 *     encode("NativeAsset".policy_id, 'hex') as policy_id,
 *     encode("NativeAsset".asset_name, 'hex') as asset_name,
 *     encode("Transaction".hash, 'hex') as action_tx_id,
 *     encode("Block".hash, 'hex') as action_block_id,
 *     CASE
 *         WHEN "TransactionMetadata".payload = NULL THEN NULL
 *         ELSE encode("TransactionMetadata".payload, 'hex')
 *         END AS action_tx_metadata,
 *     "Block".slot as action_slot
 * FROM "AssetMint"
 *          LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
 *          JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
 *          JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *         "Block".slot > :min_slot!
 *     AND "Block".slot <= :max_slot!
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * ```
 */
export const sqlMintBurnRange = new PreparedQuery<ISqlMintBurnRangeParams,ISqlMintBurnRangeResult>(sqlMintBurnRangeIR);


/** 'SqlMintBurnRangeByPolicyIds' parameters type */
export interface ISqlMintBurnRangeByPolicyIdsParams {
  max_slot: number;
  min_slot: number;
  policy_ids: readonly (Buffer)[];
}

/** 'SqlMintBurnRangeByPolicyIds' return type */
export interface ISqlMintBurnRangeByPolicyIdsResult {
  action_block_id: string | null;
  action_slot: number;
  action_tx_id: string | null;
  action_tx_metadata: string | null;
  amount: string;
  asset_name: string | null;
  policy_id: string | null;
}

/** 'SqlMintBurnRangeByPolicyIds' query type */
export interface ISqlMintBurnRangeByPolicyIdsQuery {
  params: ISqlMintBurnRangeByPolicyIdsParams;
  result: ISqlMintBurnRangeByPolicyIdsResult;
}

const sqlMintBurnRangeByPolicyIdsIR: any = {"usedParamSet":{"min_slot":true,"max_slot":true,"policy_ids":true},"params":[{"name":"policy_ids","required":true,"transform":{"type":"array_spread"},"locs":[{"a":874,"b":885}]},{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":793,"b":802}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":828,"b":837}]}],"statement":"SELECT\n    \"AssetMint\".amount as amount,\n    encode(\"NativeAsset\".policy_id, 'hex') as policy_id,\n    encode(\"NativeAsset\".asset_name, 'hex') as asset_name,\n    encode(\"Transaction\".hash, 'hex') as action_tx_id,\n    encode(\"Block\".hash, 'hex') as action_block_id,\n    CASE\n        WHEN \"TransactionMetadata\".payload = NULL THEN NULL\n        ELSE encode(\"TransactionMetadata\".payload, 'hex')\n        END AS action_tx_metadata,\n    \"Block\".slot as action_slot\nFROM \"AssetMint\"\n         LEFT JOIN \"TransactionMetadata\" ON \"TransactionMetadata\".id = \"AssetMint\".tx_id\n         JOIN \"NativeAsset\" ON \"NativeAsset\".id = \"AssetMint\".asset_id\n         JOIN \"Transaction\" ON \"Transaction\".id = \"AssetMint\".tx_id\n         JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\nWHERE\n        \"Block\".slot > :min_slot!\n    AND \"Block\".slot <= :max_slot!\n    AND \"NativeAsset\".policy_id IN :policy_ids!\nORDER BY (\"Block\".height, \"Transaction\".tx_index) ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 *     "AssetMint".amount as amount,
 *     encode("NativeAsset".policy_id, 'hex') as policy_id,
 *     encode("NativeAsset".asset_name, 'hex') as asset_name,
 *     encode("Transaction".hash, 'hex') as action_tx_id,
 *     encode("Block".hash, 'hex') as action_block_id,
 *     CASE
 *         WHEN "TransactionMetadata".payload = NULL THEN NULL
 *         ELSE encode("TransactionMetadata".payload, 'hex')
 *         END AS action_tx_metadata,
 *     "Block".slot as action_slot
 * FROM "AssetMint"
 *          LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
 *          JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
 *          JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
 *          JOIN "Block" ON "Transaction".block_id = "Block".id
 * WHERE
 *         "Block".slot > :min_slot!
 *     AND "Block".slot <= :max_slot!
 *     AND "NativeAsset".policy_id IN :policy_ids!
 * ORDER BY ("Block".height, "Transaction".tx_index) ASC
 * ```
 */
export const sqlMintBurnRangeByPolicyIds = new PreparedQuery<ISqlMintBurnRangeByPolicyIdsParams,ISqlMintBurnRangeByPolicyIdsResult>(sqlMintBurnRangeByPolicyIdsIR);


