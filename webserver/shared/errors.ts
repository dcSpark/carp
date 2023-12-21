export enum ErrorCodes {
  // we explicitly add the numbers to this enum
  // that way removing an entry in the future isn't a breaking change
  AddressLimitExceeded = 0,
  IncorrectAddressFormat = 1,
  BlockHashNotFound = 2,
  PageStartNotFound = 3,
  UtxoLimitExceeded = 4,
  IncorrectFormat = 5,
  BlockOffsetLimit = 6,
  OffsetBlockNotFound = 7,
  AssetLimitExceeded = 8,
  CredentialLimitExceeded = 9,
  AssetPairLimitExceeded = 10,
  PoolsLimitExceeded = 11,
  SlotRangeLimitExceeded = 12,
  AssetsLimitExceeded = 12,
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
  IncorrectFormat: {
    code: ErrorCodes.IncorrectFormat,
    prefix: "Incorrectly formatted data found.",
    detailsGen: (details: object) => JSON.stringify(details),
  },
  BlockHashNotFound: {
    code: ErrorCodes.BlockHashNotFound,
    prefix: "Block hash not found.",
    detailsGen: (details: { untilBlock: string }) =>
      `Searched block hash: ${details.untilBlock}`,
  },
  PageStartNotFound: {
    code: ErrorCodes.PageStartNotFound,
    prefix: "Combination of block and transaction not found.",
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
  AssetLimitExceeded: {
    code: ErrorCodes.AssetLimitExceeded,
    prefix: "Exceeded request <policy, asset> pair limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  CredentialLimitExceeded: {
    code: ErrorCodes.CredentialLimitExceeded,
    prefix: "Exceeded request credential limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  AssetPairLimitExceeded: {
    code: ErrorCodes.AssetPairLimitExceeded,
    prefix: "Exceeded request asset pair limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  PoolsLimitExceeded: {
    code: ErrorCodes.PoolsLimitExceeded,
    prefix: "Exceeded request pools limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  SlotRangeLimitExceeded: {
    code: ErrorCodes.SlotRangeLimitExceeded,
    prefix: "Exceeded request slot range limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
  },
  AssetsLimitExceeded: {
    code: ErrorCodes.AssetLimitExceeded,
    prefix: "Exceeded request native assets limit.",
    detailsGen: (details: { limit: number; found: number }) =>
      `Limit of ${details.limit}, found ${details.found}`,
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
