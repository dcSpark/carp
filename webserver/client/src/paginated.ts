import { query } from "./index";
import { Routes } from "./index";
import type { Pagination } from "../../shared/models/common";
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
  TxAndBlockInfo,
} from "../../shared/models/TransactionHistory";

/**
 * If you don't mind using axios,
 * you can use the paginated endpoints provided by the client
 * However this endpoint allows you to pass in your own querying library
 */
export async function paginateQuery<T extends Pagination, Response>(
  initialRequest: T,
  query: (request: T) => Promise<Response[]>,
  pageFromResponse: (resp: undefined | Response) => Pagination["after"]
): Promise<Response[]> {
  let nextRequest: T = initialRequest;
  const result: Response[] = [];
  let currentPage: Response[] = [];
  do {
    currentPage = await query(nextRequest);
    result.push(...currentPage);

    nextRequest = {
      ...nextRequest,
      after: pageFromResponse(currentPage[currentPage.length - 1]),
    };
  } while (currentPage.length === 0);

  return result;
}

export async function paginatedTransactionHistory(
  urlBase: string,
  initialRequest: Omit<TransactionHistoryRequest, "after">
): Promise<TransactionHistoryResponse> {
  const result = await paginateQuery<TransactionHistoryRequest, TxAndBlockInfo>(
    initialRequest,
    async (request) =>
      (
        await query(urlBase, Routes.transactionHistory, request)
      ).transactions,
    (resp) =>
      resp != null
        ? {
            block: resp.block.hash,
            tx: resp.transaction.hash,
          }
        : undefined
  );
  return { transactions: result };
}
