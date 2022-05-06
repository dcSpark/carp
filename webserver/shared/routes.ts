import type {
  AddressUsedRequest,
  AddressUsedResponse,
} from "./models/AddressUsed";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from "./models/TransactionHistory";

export enum Routes {
  transactionHistory = "transaction/history",
  addressUsed = "address/used",
}

export type EndpointTypes = {
  [Routes.transactionHistory]: {
    name: typeof Routes.transactionHistory;
    input: TransactionHistoryRequest;
    response: TransactionHistoryResponse;
  };
  [Routes.addressUsed]: {
    name: typeof Routes.addressUsed;
    input: AddressUsedRequest;
    response: AddressUsedResponse;
  };
};
