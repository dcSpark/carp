import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { delegationForAddress } from '../services/DelegationForAddress';
import type { DelegationForAddressResponse } from '../../../shared/models/DelegationForAddress';
import { Address } from '@dcspark/cardano-multiplatform-lib-nodejs';

const route = Routes.delegationForAddress;

@Route('delegation/address')
export class DelegationForAddressController extends Controller {
    /**
     * Returns the pool of the last delegation for this address.
     *
     * Note: the tx can be in the current epoch, so the delegation may not be in
     * effect yet.
     */
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
        const address = Address.from_bech32(requestBody.address);
        const rewardAddr = address.as_reward();
        const stakingCred = address.staking_cred();

        let credential: Buffer;

        if(rewardAddr) {
            credential = Buffer.from(rewardAddr.payment_cred().to_bytes());
            rewardAddr.free();
        }
        else if(stakingCred) {
            credential = Buffer.from(stakingCred.to_bytes());
            stakingCred.free();
        }
        else {
            address.free();

            // eslint-disable-next-line @typescript-eslint/no-unsafe-return
            return errorResponse(
                StatusCodes.UNPROCESSABLE_ENTITY,
                genErrorMessage(Errors.IncorrectAddressFormat, {
                    addresses: [requestBody.address],
                })
            );
        }

        address.free();

        const response = await tx<
            DelegationForAddressResponse
        >(pool, async dbTx => {
            const data = await delegationForAddress({
                address: credential,
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

