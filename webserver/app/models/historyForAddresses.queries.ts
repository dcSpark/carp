/** Types generated for queries found in "app/models/historyForAddresses.sql" */
import { PreparedQuery } from '@pgtyped/query';

/** 'HistoryForAddresses' parameters type */
export type IHistoryForAddressesParams = void;

/** 'HistoryForAddresses' return type */
export interface IHistoryForAddressesResult {
  block_id: number;
  hash: Buffer;
  id: string;
  is_valid: boolean;
  payload: Buffer;
  tx_index: number;
}

/** 'HistoryForAddresses' query type */
export interface IHistoryForAddressesQuery {
  params: IHistoryForAddressesParams;
  result: IHistoryForAddressesResult;
}

const historyForAddressesIR: any = {"name":"HistoryForAddresses","params":[],"usedParamSet":{},"statement":{"body":"SELECT * from \"Transaction\" LIMIT 100","loc":{"a":32,"b":68,"line":2,"col":0}}};

/**
 * Query generated from SQL:
 * ```
 * SELECT * from "Transaction" LIMIT 100
 * ```
 */
export const historyForAddresses = new PreparedQuery<IHistoryForAddressesParams,IHistoryForAddressesResult>(historyForAddressesIR);


