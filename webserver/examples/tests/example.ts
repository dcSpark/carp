import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import sortBy from "lodash/sortBy";
import { bech32 } from "bech32";
import Cip5 from "@dcspark/cip5-js";
import {
  Pagination,
  RelationFilterType,
} from "@dcspark/carp-client/shared/models/common";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
  TxAndBlockInfo,
} from "@dcspark/carp-client/shared/models/TransactionHistory";

const urlBase = "http://localhost:3000";

async function paginationByTx<T extends Pagination, Response>(
  request: T
): Promise<TransactionHistoryResponse> {
  let nextRequest: T = request;
  const result: Response[] = [];
  let currentPage: Response[] = [];
  do {
    currentPage = (await query<Routes.transactionHistory>(urlBase, request))
      .transactions;
    result.push(...currentPage);

    nextRequest = {
      ...nextRequest,
      after:
        currentPage.length > 0
          ? {
              block: currentPage[currentPage.length - 1].block.hash,
              tx: currentPage[currentPage.length - 1].transaction.hash,
            }
          : undefined,
    };
  } while (currentPage.length === 0);

  return { transactions: result };
}

async function paginationTxHistory(
  request: Omit<TransactionHistoryRequest, "after">
): Promise<TransactionHistoryResponse> {
  let nextRequest: TransactionHistoryRequest = request;
  const result: TxAndBlockInfo[] = [];
  let currentPage: TxAndBlockInfo[] = [];
  do {
    currentPage = (await query<Routes.transactionHistory>(urlBase, request))
      .transactions;
    result.push(...currentPage);

    nextRequest = {
      ...nextRequest,
      after:
        currentPage.length > 0
          ? {
              block: currentPage[currentPage.length - 1].block.hash,
              tx: currentPage[currentPage.length - 1].transaction.hash,
            }
          : undefined,
    };
  } while (currentPage.length === 0);

  return { transactions: result };
}

async function getHistoryForAddress(bech32Address: string) {
  const bestBlock = await query<Routes.blockLatest>(urlBase, {
    // the higher you make this, the less you have to worry about rollbacks
    // but also the slower your app will react to new transactions by the user
    // you can look into projects like Cardano multiverse-rs to optimize this number
    offset: 3,
  });

  const originalAddr =
    "addr1q9ugzr9rh9er9798vt7y2npf2526lj9te3cr4ucv80cg0as6fmujje8aavgmtn4h6h3uwj7d5zds5uzhgv556lm9vmfsk7xyk5";
  const wasmAddr = cml.Address.from_bech32(originalAddr);
  const paymentKey = wasmAddr.as_base()?.payment_cred();
  if (paymentKey == null) throw new Error();

  const stakingKey = wasmAddr.as_base()?.stake_cred();
  if (stakingKey == null) throw new Error();

  const result = await query<Routes.transactionHistory>(urlBase, {
    addresses: [
      Buffer.from(paymentKey.to_bytes()).toString("hex"),
      Buffer.from(stakingKey.to_bytes()).toString("hex"),
    ],
    untilBlock: bestBlock.block.hash,
  });
  expect(result.transactions).be.empty;
}
