import type {
  AddressUsedRequest,
  AddressUsedResponse,
} from "./models/AddressUsed";
import { BlockLatestRequest, BlockLatestResponse } from "./models/BlockLatest";
import { Cip25Response, PolicyIdAssetMapType } from "./models/PolicyIdAssetMap";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from "./models/TransactionHistory";
import type {
  TransactionOutputRequest,
  TransactionOutputResponse,
} from "./models/TransactionOutput";

export enum Routes {
  transactionHistory = "transaction/history",
  transactionOutput = "transaction/output",
  addressUsed = "address/used",
  blockLatest = "block/latest",
  metadataNft = "metadata/nft",
}

export type EndpointTypes = {
  [Routes.transactionHistory]: {
    name: typeof Routes.transactionHistory;
    input: TransactionHistoryRequest;
    response: TransactionHistoryResponse;
  };
  [Routes.transactionOutput]: {
    name: typeof Routes.transactionOutput;
    input: TransactionOutputRequest;
    response: TransactionOutputResponse;
  };
  [Routes.addressUsed]: {
    name: typeof Routes.addressUsed;
    input: AddressUsedRequest;
    response: AddressUsedResponse;
  };
  [Routes.blockLatest]: {
    name: typeof Routes.blockLatest;
    input: BlockLatestRequest;
    response: BlockLatestResponse;
  };
  [Routes.metadataNft]: {
    name: typeof Routes.metadataNft;
    input: PolicyIdAssetMapType;
    response: Cip25Response;
  };
};
