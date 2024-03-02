import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { genErrorMessage, type ErrorShape, Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { delegationsForPool } from '../services/DelegationForPool';
import type { DelegationForPoolResponse } from '../../../shared/models/DelegationForPool';
import { POOL_DELEGATION_LIMIT } from '../../../shared/constants';
import {
  adjustToSlotLimits,
  resolvePageStart,
  resolveUntilTransaction,
} from '../services/PaginationService';
import { slotBoundsPagination } from '../models/pagination/slotBoundsPagination.queries';
import { expectType } from 'tsd';

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

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const response = await tx<ErrorShape | DelegationForPoolResponse>(pool, async dbTx => {
      const [until, pageStart, slotBounds] = await Promise.all([
        resolveUntilTransaction({
          block_hash: Buffer.from(requestBody.untilBlock, 'hex'),
          dbTx,
        }),
        requestBody.after == null
          ? Promise.resolve(undefined)
          : resolvePageStart({
              after_block: Buffer.from(requestBody.after.block, 'hex'),
              after_tx: Buffer.from(requestBody.after.tx, 'hex'),
              dbTx,
            }),
        !requestBody.slotLimits
          ? Promise.resolve(undefined)
          : slotBoundsPagination.run(
              { low: requestBody.slotLimits.from, high: requestBody.slotLimits.to },
              dbTx
            ),
      ]);

      if (until == null) {
        return genErrorMessage(Errors.BlockHashNotFound, {
          untilBlock: requestBody.untilBlock,
        });
      }
      if (requestBody.after != null && pageStart == null) {
        return genErrorMessage(Errors.PageStartNotFound, {
          blockHash: requestBody.after.block,
          txHash: requestBody.after.tx,
        });
      }

      const pageStartWithSlot = adjustToSlotLimits(
        pageStart,
        until,
        requestBody.slotLimits,
        slotBounds
      );

      const response = await delegationsForPool({
        pools: requestBody.pools.map(poolId => Buffer.from(poolId, 'hex')),
        after: pageStartWithSlot?.tx_id || 0,
        until: until.tx_id,
        limit: requestBody.limit || POOL_DELEGATION_LIMIT.DEFAULT_PAGE_SIZE,
        dbTx,
      });

      return response.map(x => ({
        txId: x.tx_id,
        block: x.block,
        payload: x.payload as DelegationForPoolResponse[0]['payload'],
      }));
    });

    if ('code' in response) {
      expectType<Equals<typeof response, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, response);
    }

    return response;
  }
}
