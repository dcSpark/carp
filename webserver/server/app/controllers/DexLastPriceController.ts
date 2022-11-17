import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { DEX_PRICE_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import type { DexLastPriceResponse } from '../../../shared/models/DexLastPrice';
import { dexLastPrice } from '../services/DexLastPrice';


const route = Routes.dexLastPrice;

@Route('dex/last-price')
export class DexLastPriceController extends Controller {
    /**
     * Gets the swap prices for the given liquidity pool and asset pairs.
     */
    @SuccessResponse(`${StatusCodes.OK}`)
    @Post()
    public async dexLastPrice(
        @Body()
        requestBody: EndpointTypes[typeof route]['input'],
        @Res()
        errorResponse: TsoaResponse<
            StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
            ErrorShape
        >
    ): Promise<EndpointTypes[typeof route]['response']> {
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
        const lastPrices = await tx<ErrorShape | DexLastPriceResponse>(
            pool,
            async dbTx => {
                  return await dexLastPrice({         
                    dbTx,           
                    assetPairs: requestBody.assetPairs,
                    type: requestBody.type
                });
            }
        );
        if ('code' in lastPrices) {
            expectType<Equals<typeof lastPrices, ErrorShape>>(true);
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return
            return errorResponse(StatusCodes.CONFLICT, lastPrices);
        }

        return lastPrices;
    }
}
