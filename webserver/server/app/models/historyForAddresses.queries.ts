/** Types generated for queries found in "app/models/historyForAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

export type BufferArray = (Buffer)[];

/** 'HistoryForAddresses' parameters type */
export interface IHistoryForAddressesParams {
  credentials: BufferArray | null | void;
}

/** 'HistoryForAddresses' return type */
export interface IHistoryForAddressesResult {
  block_hash: Buffer;
  epoch: number;
  era: number;
  hash: Buffer;
  height: number;
  id: string;
  is_valid: boolean;
  payload: Buffer;
  slot: number;
  tx_index: number;
}

/** 'HistoryForAddresses' query type */
export interface IHistoryForAddressesQuery {
  params: IHistoryForAddressesParams;
  result: IHistoryForAddressesResult;
}

const historyForAddressesIR: any = {"name":"HistoryForAddresses","params":[{"name":"credentials","required":false,"transform":{"type":"scalar"},"codeRefs":{"used":[{"a":632,"b":642,"line":16,"col":49}]}}],"usedParamSet":{"credentials":true},"statement":{"body":"SELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".hash AS block_hash,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM \"StakeCredential\"\n      INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n      INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n      INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n      WHERE \"StakeCredential\".credential = ANY (:credentials)\n      ORDER BY\n        \"Block\".height ASC,\n        \"Transaction\".tx_index ASC\n      LIMIT 100","loc":{"a":32,"b":737,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT "Transaction".id,
 *         "Transaction".payload,
 *         "Transaction".hash,
 *         "Transaction".tx_index,
 *         "Transaction".is_valid,
 *         "Block".hash AS block_hash,
 *         "Block".epoch,
 *         "Block".slot,
 *         "Block".era,
 *         "Block".height
 *       FROM "StakeCredential"
 *       INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *       INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *       INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *       WHERE "StakeCredential".credential = ANY (:credentials)
 *       ORDER BY
 *         "Block".height ASC,
 *         "Transaction".tx_index ASC
 *       LIMIT 100
 * ```
 */
export const historyForAddresses = new PreparedQuery<IHistoryForAddressesParams,IHistoryForAddressesResult>(historyForAddressesIR);


