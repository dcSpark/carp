import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getAddressTypes } from '../models/utils';
import { delegationForAddress } from '../services/Delegation';
import { DelegationForAddressResponse } from '../../../shared/models/DelegationForAddress';

const route = Routes.delegationForAddress;

@Route('delegation/address')
export class DelegationForAddressController extends Controller {
    @SuccessResponse(`${StatusCodes.OK}`)
    @Post()
    public async delegationForAddress(
        @Body()
        requestBody: EndpointTypes[typeof route]['input'],
        @Res()
        errorResponse: TsoaResponse<
            StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
            ErrorShape
        >
    ): Promise<EndpointTypes[typeof route]['response']> {
        const addressTypes = getAddressTypes([requestBody.address]);

        if (addressTypes.invalid.length > 0) {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return
            return errorResponse(
                StatusCodes.UNPROCESSABLE_ENTITY,
                genErrorMessage(Errors.IncorrectAddressFormat, {
                    addresses: addressTypes.invalid,
                })
            );
        }

        const response = await tx<
            DelegationForAddressResponse
        >(pool, async dbTx => {
            const data = await delegationForAddress({
                address: addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex'))[0],
                until: requestBody.until,
                dbTx
            });

            return {
                pool: data ? data.pool : null,
                txId: data ? data.tx_id : null,
            }
        });

        return response;
    }
}

