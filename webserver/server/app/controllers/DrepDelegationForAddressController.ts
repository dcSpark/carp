import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { Address, RewardAddress } from '@dcspark/cardano-multiplatform-lib-nodejs';
import { DrepDelegationForAddressResponse } from '../../../shared/models/DelegationForAddress';
import { drepDelegationForAddress } from '../services/DrepDelegationForAddress';

const route = Routes.drepDelegationForAddress;

@Route('delegation/drep/address')
export class DrepDelegationForAddressController extends Controller {
    /**
     * Returns the drep of the last delegation for this address.
     */
    @SuccessResponse(`${StatusCodes.OK}`)
    @Post()
    public async drepDelegationForAddress(
        @Body()
        requestBody: EndpointTypes[typeof route]['input'],
        @Res()
        errorResponse: TsoaResponse<
            StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
            ErrorShape
        >
    ): Promise<EndpointTypes[typeof route]['response']> {
        const address = Address.from_bech32(requestBody.address);
        const rewardAddr = RewardAddress.from_address(address);
        const stakingCred = address.staking_cred();

        let credential: Buffer;

        if(rewardAddr) {
            credential = Buffer.from(rewardAddr.payment().to_cbor_bytes());
        }
        else if(stakingCred) {
            credential = Buffer.from(stakingCred.to_cbor_bytes());
        }
        else {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return
            return errorResponse(
                StatusCodes.UNPROCESSABLE_ENTITY,
                genErrorMessage(Errors.IncorrectAddressFormat, {
                    addresses: [requestBody.address],
                })
            );
        }

        const response = await tx<
            DrepDelegationForAddressResponse
        >(pool, async dbTx => {
            const data = await drepDelegationForAddress({
                address: credential,
                until: requestBody.until,
                dbTx
            });

            return {
                drep: data ? data.drep : null,
                txId: data ? data.tx_id : null,
            }
        });

        return response;
    }
}

