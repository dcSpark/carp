import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getLatestBlock } from '../services/BlockLatestService';
import { BLOCK_LIMIT } from '../../../shared/constants';

const route = Routes.blockLatest;

@Route('block/latest')
export class BlockLatestController extends Controller {
  /**
   * Get the latest block. Useful for checking synchronization process and pagination
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async blockLatest(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    const normalizedOffset = Math.abs(requestBody.offset);
    if (normalizedOffset > BLOCK_LIMIT.OFFSET) {
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.BlockOffsetLimit, {
          offset: requestBody.offset,
          limit: BLOCK_LIMIT.OFFSET,
        })
      );
    }
    const latestBlock = await getLatestBlock({
      dbTx: pool,
      offset: normalizedOffset,
    });
    if (latestBlock == null) {
      return errorResponse(
        StatusCodes.PRECONDITION_REQUIRED,
        genErrorMessage(Errors.OffsetBlockNotFound, {
          offset: requestBody.offset,
        })
      );
    }
    return latestBlock;
  }
}
