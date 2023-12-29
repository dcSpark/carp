/** Types generated for queries found in "app/models/asset/assetUtxos.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'AssetUtxos' parameters type */
export interface IAssetUtxosParams {
  fingerprints: readonly (Buffer | null | void)[];
  max_slot: number;
  min_slot: number;
  policyIds: readonly (Buffer | null | void)[];
}

/** 'AssetUtxos' return type */
export interface IAssetUtxosResult {
  address_raw: Buffer;
  amount: string | null;
  asset_name: Buffer;
  cip14_fingerprint: Buffer;
  output_index: number;
  output_tx_hash: string | null;
  policy_id: Buffer;
  slot: number;
  tx_hash: string | null;
}

/** 'AssetUtxos' query type */
export interface IAssetUtxosQuery {
  params: IAssetUtxosParams;
  result: IAssetUtxosResult;
}

const assetUtxosIR: any = {"usedParamSet":{"fingerprints":true,"policyIds":true,"min_slot":true,"max_slot":true},"params":[{"name":"fingerprints","required":false,"transform":{"type":"array_spread"},"locs":[{"a":731,"b":743}]},{"name":"policyIds","required":false,"transform":{"type":"array_spread"},"locs":[{"a":777,"b":786}]},{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":811,"b":820}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":843,"b":852}]}],"statement":"SELECT ENCODE(TXO.HASH,\n        'hex') OUTPUT_TX_HASH,\n    \"TransactionOutput\".OUTPUT_INDEX,\n\t\"NativeAsset\".CIP14_FINGERPRINT,\n\t\"NativeAsset\".POLICY_ID,\n\t\"NativeAsset\".ASSET_NAME,\n\t\"AssetUtxo\".AMOUNT,\n\t\"Block\".SLOT,\n\tENCODE(\"Transaction\".HASH,\n        'hex') TX_HASH,\n\t\"Address\".PAYLOAD ADDRESS_RAW\nFROM \"AssetUtxo\"\nJOIN \"Transaction\" ON \"AssetUtxo\".TX_ID = \"Transaction\".ID\nJOIN \"TransactionOutput\" ON \"AssetUtxo\".UTXO_ID = \"TransactionOutput\".ID\nJOIN \"Transaction\" TXO ON \"TransactionOutput\".TX_ID = TXO.ID\nJOIN \"Address\" ON \"Address\".id = \"TransactionOutput\".address_id\nJOIN \"NativeAsset\" ON \"AssetUtxo\".ASSET_ID = \"NativeAsset\".ID\nJOIN \"Block\" ON \"Transaction\".BLOCK_ID = \"Block\".ID\nWHERE \n\t(\"NativeAsset\".CIP14_FINGERPRINT IN :fingerprints\n\t\tOR \"NativeAsset\".POLICY_ID IN :policyIds\n\t) AND\n\t\"Block\".SLOT > :min_slot! AND\n\t\"Block\".SLOT <= :max_slot!\nORDER BY \"Transaction\".ID, \"AssetUtxo\".ID ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT ENCODE(TXO.HASH,
 *         'hex') OUTPUT_TX_HASH,
 *     "TransactionOutput".OUTPUT_INDEX,
 * 	"NativeAsset".CIP14_FINGERPRINT,
 * 	"NativeAsset".POLICY_ID,
 * 	"NativeAsset".ASSET_NAME,
 * 	"AssetUtxo".AMOUNT,
 * 	"Block".SLOT,
 * 	ENCODE("Transaction".HASH,
 *         'hex') TX_HASH,
 * 	"Address".PAYLOAD ADDRESS_RAW
 * FROM "AssetUtxo"
 * JOIN "Transaction" ON "AssetUtxo".TX_ID = "Transaction".ID
 * JOIN "TransactionOutput" ON "AssetUtxo".UTXO_ID = "TransactionOutput".ID
 * JOIN "Transaction" TXO ON "TransactionOutput".TX_ID = TXO.ID
 * JOIN "Address" ON "Address".id = "TransactionOutput".address_id
 * JOIN "NativeAsset" ON "AssetUtxo".ASSET_ID = "NativeAsset".ID
 * JOIN "Block" ON "Transaction".BLOCK_ID = "Block".ID
 * WHERE 
 * 	("NativeAsset".CIP14_FINGERPRINT IN :fingerprints
 * 		OR "NativeAsset".POLICY_ID IN :policyIds
 * 	) AND
 * 	"Block".SLOT > :min_slot! AND
 * 	"Block".SLOT <= :max_slot!
 * ORDER BY "Transaction".ID, "AssetUtxo".ID ASC
 * ```
 */
export const assetUtxos = new PreparedQuery<IAssetUtxosParams,IAssetUtxosResult>(assetUtxosIR);


