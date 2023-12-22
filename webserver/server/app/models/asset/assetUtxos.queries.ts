/** Types generated for queries found in "app/models/asset/assetUtxos.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'AssetUtxos' parameters type */
export interface IAssetUtxosParams {
  fingerprints: readonly (Buffer)[];
  max_slot: number;
  min_slot: number;
}

/** 'AssetUtxos' return type */
export interface IAssetUtxosResult {
  address_raw: Buffer;
  amount: string | null;
  cip14_fingerprint: Buffer;
  output_index: number;
  output_tx_hash: string | null;
  slot: number;
  tx_hash: string | null;
}

/** 'AssetUtxos' query type */
export interface IAssetUtxosQuery {
  params: IAssetUtxosParams;
  result: IAssetUtxosResult;
}

const assetUtxosIR: any = {"usedParamSet":{"fingerprints":true,"min_slot":true,"max_slot":true},"params":[{"name":"fingerprints","required":true,"transform":{"type":"array_spread"},"locs":[{"a":677,"b":690}]},{"name":"min_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":712,"b":721}]},{"name":"max_slot","required":true,"transform":{"type":"scalar"},"locs":[{"a":744,"b":753}]}],"statement":"SELECT ENCODE(TXO.HASH,\n        'hex') OUTPUT_TX_HASH,\n    \"TransactionOutput\".OUTPUT_INDEX,\n\t\"NativeAsset\".CIP14_FINGERPRINT,\n\t\"AssetUtxo\".AMOUNT,\n\t\"Block\".SLOT,\n\tENCODE(\"Transaction\".HASH,\n        'hex') TX_HASH,\n\t\"Address\".PAYLOAD ADDRESS_RAW\nFROM \"AssetUtxo\"\nJOIN \"Transaction\" ON \"AssetUtxo\".TX_ID = \"Transaction\".ID\nJOIN \"TransactionOutput\" ON \"AssetUtxo\".UTXO_ID = \"TransactionOutput\".ID\nJOIN \"Transaction\" TXO ON \"TransactionOutput\".TX_ID = TXO.ID\nJOIN \"Address\" ON \"Address\".id = \"TransactionOutput\".address_id\nJOIN \"NativeAsset\" ON \"AssetUtxo\".ASSET_ID = \"NativeAsset\".ID\nJOIN \"Block\" ON \"Transaction\".BLOCK_ID = \"Block\".ID\nWHERE \n\t\"NativeAsset\".CIP14_FINGERPRINT IN :fingerprints! AND\n\t\"Block\".SLOT > :min_slot! AND\n\t\"Block\".SLOT <= :max_slot!\nORDER BY \"Transaction\".ID ASC"};

/**
 * Query generated from SQL:
 * ```
 * SELECT ENCODE(TXO.HASH,
 *         'hex') OUTPUT_TX_HASH,
 *     "TransactionOutput".OUTPUT_INDEX,
 * 	"NativeAsset".CIP14_FINGERPRINT,
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
 * 	"NativeAsset".CIP14_FINGERPRINT IN :fingerprints! AND
 * 	"Block".SLOT > :min_slot! AND
 * 	"Block".SLOT <= :max_slot!
 * ORDER BY "Transaction".ID ASC
 * ```
 */
export const assetUtxos = new PreparedQuery<IAssetUtxosParams,IAssetUtxosResult>(assetUtxosIR);


