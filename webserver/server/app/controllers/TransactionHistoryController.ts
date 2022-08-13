import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { historyForAddresses, historyForCredentials } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';
import type { TransactionHistoryResponse } from '../../../shared/models/TransactionHistory';
import { ADDRESS_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilTransaction } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import sortBy from 'lodash/sortBy';
import { getAddressTypes } from '../models/utils';
import { RelationFilterType } from '../../../shared/models/common';

const route = Routes.transactionHistory;

@Route('transaction/history')
export class TransactionHistoryController extends Controller {
  /**
   * Ordered by `<block.height, transaction.tx_index>`
   *
   * Note: this endpoint only returns txs that are in a block. Use another tool to see mempool for txs not in a block
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async transactionHistory(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.CONFLICT | StatusCodes.BAD_REQUEST | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.addresses.length > ADDRESS_LIMIT.REQUEST) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AddressLimitExceeded, {
          limit: ADDRESS_LIMIT.REQUEST,
          found: requestBody.addresses.length,
        })
      );
    }
    const addressTypes = getAddressTypes(requestBody.addresses);
    if (addressTypes.invalid.length > 0) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.UNPROCESSABLE_ENTITY,
        genErrorMessage(Errors.IncorrectAddressFormat, {
          addresses: addressTypes.invalid,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const cardanoTxs = await tx<
      ErrorShape | [TransactionHistoryResponse, TransactionHistoryResponse]
    >(pool, async dbTx => {
      const [until, pageStart] = await Promise.all([
        resolveUntilTransaction({
          block_hash: Buffer.from(requestBody.untilBlock, 'hex'),
          dbTx,
        }),
        requestBody.after == null
          ? Promise.resolve(undefined)
          : resolvePageStart({
              after_block: Buffer.from(requestBody.after.block, 'hex'),
              after_tx: Buffer.from(requestBody.after.tx, 'hex'),
              dbTx,
            }),
      ]);
      if (until == null) {
        return genErrorMessage(Errors.BlockHashNotFound, {
          block: requestBody.untilBlock,
        });
      }
      if (requestBody.after != null && pageStart == null) {
        return genErrorMessage(Errors.PageStartNotFound, {
          blockHash: requestBody.after.block,
          txHash: requestBody.after.tx,
        });
      }

      const commonRequest = {
        after: pageStart,
        limit: requestBody.limit ?? ADDRESS_LIMIT.RESPONSE,
        until,
        dbTx,
      };
      const result = await Promise.all([
        historyForCredentials({
          stakeCredentials: addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex')),
          relationFilter: requestBody.relationFilter ?? RelationFilterType.NO_FILTER,
          ...commonRequest,
        }),
        historyForAddresses({
          addresses: [
            ...addressTypes.exactAddress.map(addr => Buffer.from(addr, 'hex')),
            ...addressTypes.exactLegacyAddress.map(addr => Buffer.from(addr, 'hex')),
          ],
          ...commonRequest,
        }),
      ]);
      return result;
    });
    if ('code' in cardanoTxs) {
      expectType<Equals<typeof cardanoTxs, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, cardanoTxs);
    }

    const mergedTxs = sortBy(
      [...cardanoTxs[0].transactions, ...cardanoTxs[1].transactions],
      [tx => tx.block.height, tx => tx.block.indexInBlock]
    );

    return {
      transactions:
        mergedTxs.length > ADDRESS_LIMIT.RESPONSE
          ? mergedTxs.slice(0, ADDRESS_LIMIT.RESPONSE)
          : mergedTxs,
    };
  }
}
