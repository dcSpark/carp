import type { PaginationType } from "../server/app/services/PaginationService";

export enum ErrorCodes {
  // we explicitly add the numbers to this enum
  // that way removing an entry in the future isn't a breaking change
  AddressLimitExceeded = 0,
  IncorrectAddressFormat = 1,
  BlockHashNotFound = 2,
  PageStartNotFound = 3,
  UtxoLimitExceeded = 4,
  IncorrectTxHashFormat = 5,
  BlockOffsetLimit = 6,
  OffsetBlockNotFound = 7,
}

export type ErrorShape = {
  code: number;
  reason: string;
};

export const Errors = {
  AddressLimitExceeded: {
    code: ErrorCodes.AddressLimitExceeded,
    prefix: "Exceeded request address limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  UtxoLimitExceeded: {
    code: ErrorCodes.UtxoLimitExceeded,
    prefix: "Exceeded request utxo limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  IncorrectAddressFormat: {
    code: ErrorCodes.IncorrectAddressFormat,
    prefix: "Incorrectly formatted addresses found.",
    detailsGen: (details: { addresses: string[] }) =>
      JSON.stringify(details.addresses),
  },
  IncorrectTxHashFormat: {
    code: ErrorCodes.IncorrectTxHashFormat,
    prefix: "Incorrectly formatted transaction hash found.",
    detailsGen: (details: { txHash: string[] }) =>
      JSON.stringify(details.txHash),
  },
  BlockHashNotFound: {
    code: ErrorCodes.BlockHashNotFound,
    prefix: "Block hash not found.",
    detailsGen: (details: { untilBlock: string }) =>
      `Searched block hash: ${details.untilBlock}`,
  },
  PageStartNotFound: {
    code: ErrorCodes.PageStartNotFound,
    prefix: "After block and/or transaction not found.",
    detailsGen: (details: { blockHash: string; txHash: string }) =>
      `Searched block hash ${details.blockHash} and tx hash ${details.txHash}`,
  },
  BlockOffsetLimit: {
    code: ErrorCodes.BlockOffsetLimit,
    prefix: "Block offset exceeded the limit.",
    detailsGen: (details: { offset: number; limit: number }) =>
      `Offset used was ${details.offset}, but limit is ${details.limit}`,
  },
  OffsetBlockNotFound: {
    code: ErrorCodes.OffsetBlockNotFound,
    prefix:
      "Block not found at offset. Are you sure your database is synchronized?",
    detailsGen: (details: { offset: number }) =>
      `Offset used was ${details.offset}`,
  },
} as const;

export function genErrorMessage<T extends typeof Errors[keyof typeof Errors]>(
  type: T,
  details: Parameters<T["detailsGen"]>[0]
): {
  code: T["code"];
  reason: string;
} {
  const generatedDetails = type.detailsGen(details as any);
  return {
    code: type.code,
    reason:
      generatedDetails.length === 0
        ? type.prefix
        : `${type.prefix} ${generatedDetails}`,
  };
}
