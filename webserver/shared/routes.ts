import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from "./models/TransactionHistory";

export enum Routes {
  txsForAddresses = "txsForAddresses",
}

export type EndpointTypes = {
  [Routes.txsForAddresses]: {
    name: typeof Routes.txsForAddresses;
    input: TransactionHistoryRequest;
    response: TransactionHistoryResponse;
  };
};
