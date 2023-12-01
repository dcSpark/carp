import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { projectedNftRange } from '../services/ProjectedNftRange';
import type {ProjectedNftRangeResponse} from '../../../shared/models/ProjectedNftRange';

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
                ownerAddress: data.owner_address,
                previousTxHash: data.previous_tx_hash,
                previousTxOutputIndex: data.previous_tx_output_index != null ? parseInt(data.previous_tx_output_index) : null,
                actionTxId: data.action_tx_id,
                actionOutputIndex: data.action_output_index,
                asset: data.asset,
                amount: parseInt(data.amount),
                status: data.status,
                plutusDatum: data.plutus_datum,
                actionSlot: data.action_slot,
            }));
        });

        return response;
    }
}