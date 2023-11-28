import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { genErrorMessage, type ErrorShape, Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { delegationsForPool } from '../services/DelegationForPool';
import { DelegationForPoolResponse } from '../../../shared/models/DelegationForPool';
import { POOL_DELEGATION_LIMIT } from '../../../shared/constants';

const route = Routes.delegationForPool;

@Route('delegation/pool')
export class DelegationForPoolController extends Controller {
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async delegationForPool(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.pools.length > POOL_DELEGATION_LIMIT.POOLS) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.PoolsLimitExceeded, {
          limit: POOL_DELEGATION_LIMIT.POOLS,
          found: requestBody.pools.length,
        })
      );
    }

    const slotRangeSize = requestBody.range.maxSlot - requestBody.range.minSlot;
    if (slotRangeSize > POOL_DELEGATION_LIMIT.SLOT_RANGE) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.SlotRangeLimitExceeded, {
          limit: POOL_DELEGATION_LIMIT.SLOT_RANGE,
          found: slotRangeSize,
        })
      );
    }

    const response = await tx<DelegationForPoolResponse>(pool, async dbTx => {
      const data = await delegationsForPool({
        pools: requestBody.pools.map(poolId => Buffer.from(poolId, 'hex')),
        range: requestBody.range,
        dbTx,
      });

      return data.map(data => ({
        credential: data.credential as string,
        pool: data.pool,
        txId: data.tx_id as string,
        slot: data.slot,
      }));
    });

    return response;
  }
}
