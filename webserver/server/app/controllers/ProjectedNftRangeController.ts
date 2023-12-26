import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { projectedNftRange, projectedNftRangeByAddress } from '../services/ProjectedNftRange';
import type {ProjectedNftRangeResponse, ProjectedNftStatus} from '../../../shared/models/ProjectedNftRange';
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
        const slotRangeSize = requestBody.range.maxSlot - requestBody.range.minSlot;

        if (requestBody.address !== undefined) {
            if (slotRangeSize > PROJECTED_NFT_LIMIT.SINGLE_USER_SLOT_RANGE) {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return errorResponse(
                    StatusCodes.BAD_REQUEST,
                    genErrorMessage(Errors.SlotRangeLimitExceeded, {
                        limit: PROJECTED_NFT_LIMIT.SINGLE_USER_SLOT_RANGE,
                        found: slotRangeSize,
                    })
                );
            }

            return await this.handle_by_address_query(requestBody.address, requestBody);
        } else {
            if (slotRangeSize > PROJECTED_NFT_LIMIT.SLOT_RANGE) {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return errorResponse(
                    StatusCodes.BAD_REQUEST,
                    genErrorMessage(Errors.SlotRangeLimitExceeded, {
                        limit: PROJECTED_NFT_LIMIT.SLOT_RANGE,
                        found: slotRangeSize,
                    })
                );
            }

            return await this.handle_general_query(requestBody);
        }
    }

    async handle_general_query(
        requestBody: EndpointTypes[typeof route]['input'],
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
                policyId: data.policy_id,
                assetName: data.asset_name,
                amount: data.amount,
                status: data.status as ProjectedNftStatus | null,
                plutusDatum: data.plutus_datum,
                actionSlot: data.action_slot,
                forHowLong: data.for_how_long,
            }));
        });

        return response;
    }

    async handle_by_address_query(
        address: string,
        requestBody: EndpointTypes[typeof route]['input'],
    ): Promise<EndpointTypes[typeof route]['response']> {
        const response = await tx<
            ProjectedNftRangeResponse
        >(pool, async dbTx => {
            const data = await projectedNftRangeByAddress({
                address: address,
                range: requestBody.range,
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

        return response;
    }
}