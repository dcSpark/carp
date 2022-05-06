import type {
  AddressUsedRequest,
  AddressUsedResponse,
} from "./models/AddressUsed";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from "./models/TransactionHistory";

export enum Routes {
  txsForAddresses = "txsForAddresses",
  addressUsed = "address/used",
}

export type EndpointTypes = {
  [Routes.txsForAddresses]: {
    name: typeof Routes.txsForAddresses;
    input: TransactionHistoryRequest;
    response: TransactionHistoryResponse;
  };
  [Routes.addressUsed]: {
    name: typeof Routes.addressUsed;
    input: AddressUsedRequest;
    response: AddressUsedResponse;
  };
};
