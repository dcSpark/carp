import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { projectedNftRange, projectedNftRangeByAddress } from '../services/ProjectedNftRange';
import type {
    ProjectedNftRangeResponse,
    ProjectedNftRangeSingleResponse,
    ProjectedNftStatus
} from '../../../shared/models/ProjectedNftRange';
import {PROJECTED_NFT_LIMIT} from "../../../shared/constants";
import {Errors, genErrorMessage} from "../../../shared/errors";

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
        const after = requestBody.after != undefined ? requestBody.after : 0;
        const until = requestBody.untilSlot != undefined ? requestBody.untilSlot : Number.MAX_VALUE;
        const limit = requestBody.limit != undefined ? requestBody.limit : PROJECTED_NFT_LIMIT.MAX_LIMIT;

        if (limit > PROJECTED_NFT_LIMIT.MAX_LIMIT) {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return
            return errorResponse(
                StatusCodes.BAD_REQUEST,
                genErrorMessage(Errors.SlotRangeLimitExceeded, {
                    limit: PROJECTED_NFT_LIMIT.MAX_LIMIT,
                    found: limit,
                })
            );
        }

        let params = {
            afterSlot: after,
            untilSlot: until,
            limit: limit
        };

        if (requestBody.address !== undefined) {
            return await this.handle_by_address_query(requestBody.address, params);
        } else {
            return await this.handle_general_query(params);
        }
    }

    async handle_general_query(
        params: { afterSlot: number, untilSlot: number, limit: number },
    ): Promise<EndpointTypes[typeof route]['response']> {
        const result = await tx<
            ProjectedNftRangeSingleResponse[]
        >(pool, async dbTx => {
            const data = await projectedNftRange({
                params: params,
                dbTx
            });

            return data.map(data => ({
                ownerAddress: data.owner_address,
                previousTxHash: data.previous_tx_hash,
                previousTxOutputIndex: data.previous_tx_output_index != null ? parseInt(data.previous_tx_output_index) : null,
                actionTxId: data.action_tx_id,
                actionOutputIndex: data.action_output_index,
                policyId: data.policy_id,
                assetName: data.asset_name,
                amount: data.amount,
                status: data.status as ProjectedNftStatus | null,
                plutusDatum: data.plutus_datum,
                actionSlot: data.action_slot,
                forHowLong: data.for_how_long,
            }));
        });

        let after = undefined;

        if (result.length >= params.limit) {
            after = result[result.length - 1].actionSlot;
        }

        return {
            result: result,
            after: after,
        };
    }

    async handle_by_address_query(
        address: string,
        params: { afterSlot: number, untilSlot: number, limit: number },
    ): Promise<EndpointTypes[typeof route]['response']> {
        const result = await tx<
            ProjectedNftRangeSingleResponse[]
        >(pool, async dbTx => {
            const data = await projectedNftRangeByAddress({
                address: address,
                params: params,
                dbTx
            });

            return data.map(data => ({
                ownerAddress: data.owner_address,
                previousTxHash: data.previous_tx_hash,
                previousTxOutputIndex: data.previous_tx_output_index != null ? parseInt(data.previous_tx_output_index) : null,
                actionTxId: data.action_tx_id,
                actionOutputIndex: data.action_output_index,
                policyId: data.policy_id,
                assetName: data.asset_name,
                amount: data.amount,
                status: data.status as ProjectedNftStatus | null,
                plutusDatum: data.plutus_datum,
                actionSlot: data.action_slot,
                forHowLong: data.for_how_long,
            }));
        });

        let after = undefined;

        if (result.length >= params.limit) {
            after = result[result.length - 1].actionSlot;
        }

        return {
            result: result,
            after: after,
        };
    }
}