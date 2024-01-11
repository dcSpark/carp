/** Types generated for queries found in "app/models/asset/assetUtxos.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export type NumberOrString = number | string;

/** 'AssetUtxos' parameters type */
export interface IAssetUtxosParams {
  after_tx_id: NumberOrString;
  fingerprints: readonly (Buffer | null | void)[];
  limit: NumberOrString;
  policyIds: readonly (Buffer | null | void)[];
  until_tx_id: NumberOrString;
}

/** 'AssetUtxos' return type */
export interface IAssetUtxosResult {
  block: string;
  payload: Json;
  tx: string | null;
}

/** 'AssetUtxos' query type */
export interface IAssetUtxosQuery {
  params: IAssetUtxosParams;
  result: IAssetUtxosResult;
}

const assetUtxosIR: any = {"usedParamSet":{"fingerprints":true,"policyIds":true,"after_tx_id":true,"until_tx_id":true,"limit":true},"params":[{"name":"fingerprints","required":false,"transform":{"type":"array_spread"},"locs":[{"a":946,"b":958}]},{"name":"policyIds","required":false,"transform":{"type":"array_spread"},"locs":[{"a":992,"b":1001}]},{"name":"after_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1030,"b":1042}]},{"name":"until_tx_id","required":true,"transform":{"type":"scalar"},"locs":[{"a":1069,"b":1081}]},{"name":"limit","required":true,"transform":{"type":"scalar"},"locs":[{"a":1159,"b":1165}]}],"statement":"SELECT\n\tENCODE(\"Transaction\".HASH, 'hex') TX,\n\tENCODE(\"Block\".HASH, 'hex') AS \"block!\",\n\tjson_agg(json_build_object(\n\t\t'outputIndex', \"TransactionOutput\".OUTPUT_INDEX,\n\t\t'outputTxHash', ENCODE(TXO.HASH, 'hex'),\n\t\t'cip14Fingerprint', ENCODE(\"NativeAsset\".CIP14_FINGERPRINT, 'hex'),\n\t\t'policyId', ENCODE(\"NativeAsset\".POLICY_ID, 'hex'),\n\t\t'assetName', ENCODE(\"NativeAsset\".ASSET_NAME, 'hex'),\n\t\t'amount', \"AssetUtxo\".AMOUNT,\n\t\t'slot', \"Block\".SLOT,\n\t\t'addressRaw', ENCODE(\"Address\".PAYLOAD, 'hex')\n\t)) as \"payload!\"\nFROM \"AssetUtxo\"\nJOIN \"Transaction\" ON \"AssetUtxo\".TX_ID = \"Transaction\".ID\nJOIN \"TransactionOutput\" ON \"AssetUtxo\".UTXO_ID = \"TransactionOutput\".ID\nJOIN \"Transaction\" TXO ON \"TransactionOutput\".TX_ID = TXO.ID\nJOIN \"Address\" ON \"Address\".id = \"TransactionOutput\".address_id\nJOIN \"NativeAsset\" ON \"AssetUtxo\".ASSET_ID = \"NativeAsset\".ID\nJOIN \"Block\" ON \"Transaction\".BLOCK_ID = \"Block\".ID\nWHERE \n\t(\"NativeAsset\".CIP14_FINGERPRINT IN :fingerprints\n\t\tOR \"NativeAsset\".POLICY_ID IN :policyIds\n\t) AND\n\t\"Transaction\".id > :after_tx_id! AND\n\t\"Transaction\".id <= :until_tx_id!\nGROUP BY (\"Block\".ID, \"Transaction\".ID)\nORDER BY \"Transaction\".ID ASC\nLIMIT :limit!"};

/**
 * Query generated from SQL:
 * ```
 * SELECT
 * 	ENCODE("Transaction".HASH, 'hex') TX,
 * 	ENCODE("Block".HASH, 'hex') AS "block!",
 * 	json_agg(json_build_object(
 * 		'outputIndex', "TransactionOutput".OUTPUT_INDEX,
 * 		'outputTxHash', ENCODE(TXO.HASH, 'hex'),
 * 		'cip14Fingerprint', ENCODE("NativeAsset".CIP14_FINGERPRINT, 'hex'),
 * 		'policyId', ENCODE("NativeAsset".POLICY_ID, 'hex'),
 * 		'assetName', ENCODE("NativeAsset".ASSET_NAME, 'hex'),
 * 		'amount', "AssetUtxo".AMOUNT,
 * 		'slot', "Block".SLOT,
 * 		'addressRaw', ENCODE("Address".PAYLOAD, 'hex')
 * 	)) as "payload!"
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
 * 	"Transaction".id > :after_tx_id! AND
 * 	"Transaction".id <= :until_tx_id!
 * GROUP BY ("Block".ID, "Transaction".ID)
 * ORDER BY "Transaction".ID ASC
 * LIMIT :limit!
 * ```
 */
export const assetUtxos = new PreparedQuery<IAssetUtxosParams,IAssetUtxosResult>(assetUtxosIR);


