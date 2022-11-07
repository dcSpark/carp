import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getBlockMinter } from '../services/BlockMinterService';

const route = Routes.blockMinter;

@Route('block/minter')
export class BlockMinterController extends Controller {
  /**
   * Get the latest block. Useful for checking synchronization process and pagination
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async blockMinter(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT, ErrorShape>
  ): Promise<EndpointTypes[typeof route]['response']> {
    const minter = await getBlockMinter({
      dbTx: pool,
      ...requestBody,
    });
    if (minter == null) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.CONFLICT,
        genErrorMessage(Errors.BlockHashNotFound, {
          block: requestBody.hash,
        })
      );
    }
    return minter;
  }
}
