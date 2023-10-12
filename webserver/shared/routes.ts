import type {
  AddressUsedRequest,
  AddressUsedResponse,
} from "./models/AddressUsed";
import { BlockLatestRequest, BlockLatestResponse } from "./models/BlockLatest";
import type {
  CredentialAddressRequest,
  CredentialAddressResponse,
} from "./models/CredentialAddress";
import { DexMeanPriceRequest, DexMeanPriceResponse } from "./models/DexMeanPrice";
import { DexSwapRequest, DexSwapResponse } from "./models/DexSwap";
import { DexLastPriceRequest, DexLastPriceResponse } from "./models/DexLastPrice";
import { Cip25Response, PolicyIdAssetMapType } from "./models/PolicyIdAssetMap";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from "./models/TransactionHistory";
import type {
  TransactionOutputRequest,
  TransactionOutputResponse,
} from "./models/TransactionOutput";
import type {
  DelegationForAddressRequest,
  DelegationForAddressResponse,
} from "./models/DelegationForAddress";
import type {
  DelegationForPoolRequest,
  DelegationForPoolResponse,
} from "./models/DelegationForPool";

export enum Routes {
  transactionHistory = "transaction/history",
  transactionOutput = "transaction/output",
  addressUsed = "address/used",
  credentialAddress = "credential/address",
  blockLatest = "block/latest",
  metadataNft = "metadata/nft",
  dexMeanPrice = "dex/mean-price",
  dexSwap = "dex/swap",
  dexLastPrice = "dex/last-price",
  delegationForAddress = "delegation/address",
  delegationForPool = "delegation/pool",
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
  [Routes.credentialAddress]: {
    name: typeof Routes.credentialAddress;
    input: CredentialAddressRequest;
    response: CredentialAddressResponse;
  };
  [Routes.dexMeanPrice]: {
    name: typeof Routes.dexMeanPrice;
    input: DexMeanPriceRequest;
    response: DexMeanPriceResponse;
  };
  [Routes.dexSwap]: {
    name: typeof Routes.dexSwap;
    input: DexSwapRequest;
    response: DexSwapResponse;
  };
  [Routes.dexLastPrice]: {
    name: typeof Routes.dexLastPrice;
    input: DexLastPriceRequest;
    response: DexLastPriceResponse;
  };
  [Routes.delegationForAddress]: {
    name: typeof Routes.delegationForAddress;
    input: DelegationForAddressRequest;
    response: DelegationForAddressResponse;
  };
  [Routes.delegationForPool]: {
    name: typeof Routes.delegationForPool;
    input: DelegationForPoolRequest;
    response: DelegationForPoolResponse;
  };
};
