import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { DEX_PRICE_LIMIT } from '../../../shared/constants';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { dexLastPrice } from '../services/DexLastPrice';


const route = Routes.dexLastPrice;

@Route('dex/last-price')
export class DexLastPriceController extends Controller {
    /**
     * Gets the swap prices for the given liquidity pool and asset pairs.
     * Operation "mean" is not AVG from the last values, but the remaining amount of assets on the pool output
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

        const lastPrices = await dexLastPrice({         
            dbTx: await pool.connect(),           
            assetPairs: requestBody.assetPairs,
            type: requestBody.type
        });

        return lastPrices;
    }
}
