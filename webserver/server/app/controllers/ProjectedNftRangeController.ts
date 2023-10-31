import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { projectedNftRange } from '../services/ProjectedNftRange';
import type {ProjectedNftRangeResponse} from '../../../shared/models/ProjectedNftRange';
import {Amount, UtxoPointer} from "../../../shared/models/common";

const route = Routes.projectedNftEventsRange;

@Route('projected-nft/range')
export class ProjectedNftRangeController extends Controller {
    @SuccessResponse(`${StatusCodes.OK}`)
    @Post()
    public async projectedNftRange(
        @Body()
            requestBody: EndpointTypes[typeof route]['input'],
        @Res()
            errorResponse: TsoaResponse<
            StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
            ErrorShape
        >
    ): Promise<EndpointTypes[typeof route]['response']> {
        const response = await tx<
            ProjectedNftRangeResponse
        >(pool, async dbTx => {
            const data = await projectedNftRange({
                range: requestBody.range,
                dbTx
            });

            return data.map(data => ({
                txId: data.tx_id as string,
                outputIndex: data.output_index,
                slot: data.slot,
                asset: data.asset,
                amount: data.amount,
                status: data.status,
                plutusDatum: data.plutus_datum,
            }));
        });

        return response;
    }
}