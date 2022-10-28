import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { DEX_PRICE_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilTransaction } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getAddressTypes } from '../models/utils';
import type { DexMeanPriceResponse } from '../../../shared/models/DexMeanPrice';
import { dexMeanPrices } from '../services/DexMeanPrice';

const route = Routes.dexMeanPrice;

@Route('dex/mean-price')
export class DexMeanPriceController extends Controller {
  /**
   * Gets the mean prices for the given liquidity pool addresses and asset pairs.
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async dexMeanPrice(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.addresses.length > DEX_PRICE_LIMIT.REQUEST_ADDRESSES) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AddressLimitExceeded, {
          limit: DEX_PRICE_LIMIT.REQUEST_ADDRESSES,
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

    if (requestBody.assetPairs.length > DEX_PRICE_LIMIT.REQUEST_ASSET_PAIRS) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AssetPairLimitExceeded, {
          limit: DEX_PRICE_LIMIT.REQUEST_ASSET_PAIRS,
          found: requestBody.assetPairs.length,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const meanPrices = await tx<ErrorShape | DexMeanPriceResponse>(
      pool,
      async dbTx => {
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
            untilBlock: requestBody.untilBlock,
          });
        }
        if (requestBody.after != null && pageStart == null) {
          return genErrorMessage(Errors.PageStartNotFound, {
            blockHash: requestBody.after.block,
            txHash: requestBody.after.tx,
          });
        }

        return await dexMeanPrices({
          after: pageStart,
          until,
          dbTx,
          addresses: addressTypes.exactAddress.map(addr => Buffer.from(addr, 'hex')),
          reverseMap: addressTypes.reverseMap,
          assetPairs: requestBody.assetPairs,
          limit: requestBody.limit ?? DEX_PRICE_LIMIT.RESPONSE,
        });
      }
    );
    if ('code' in meanPrices) {
      expectType<Equals<typeof meanPrices, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, meanPrices);
    }

    return meanPrices;
  }
}
