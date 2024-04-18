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
  ProjectedNftStatus,
} from '../../../shared/models/ProjectedNftRange';
import { PROJECTED_NFT_LIMIT } from '../../../shared/constants';
import { Errors, genErrorMessage } from '../../../shared/errors';
import {
  adjustToSlotLimits,
  resolvePageStart,
  resolveUntilTransaction,
} from '../services/PaginationService';
import { slotBoundsPagination } from '../models/pagination/slotBoundsPagination.queries';
import { expectType } from 'tsd';

const route = Routes.projectedNftEventsRange;

@Route('projected-nft/range')
export class ProjectedNftRangeController extends Controller {
  /**
   * Query any projected NFT. Learn more [<u>here</u>](https://github.com/dcSpark/projected-nft-whirlpool).
   */
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
    if (requestBody.address !== undefined) {
      return await this.handle_by_address_query(requestBody.address, requestBody, errorResponse);
    } else {
      return await this.handle_general_query(requestBody, errorResponse);
    }
  }

  async handle_general_query(
    requestBody: EndpointTypes[typeof route]['input'],
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    const response = await tx<ErrorShape | ProjectedNftRangeResponse>(pool, async dbTx => {
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

      const data = await projectedNftRange({
        after: pageStartWithSlot?.tx_id || 0,
        until: until.tx_id,
        limit: requestBody.limit || PROJECTED_NFT_LIMIT.DEFAULT_PAGE_SIZE,
        dbTx,
      });

      return data.map(data => ({
        block: data.block,
        txId: data.tx_id,
        payload: (
          (data.payload as ({ [key: string]: string } & {
            actionSlot: number;
            actionOutputIndex: number;
          })[]) || []
        ).map(data => ({
          ownerAddress: data.ownerAddress,
          previousTxHash: data.previousUtxoTxHash,
          previousTxOutputIndex:
            data.previousTxOutputIndex != null ? parseInt(data.previousTxOutputIndex) : null,
          actionOutputIndex: data.actionOutputIndex,
          policyId: data.policyId,
          assetName: data.assetName,
          amount: data.amount,
          status: data.status as ProjectedNftStatus,
          plutusDatum: data.plutusDatum,
          actionSlot: data.actionSlot,
          forHowLong: data.forHowLong,
        })),
      }));
    });

    if ('code' in response) {
      expectType<Equals<typeof response, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, response);
    }

    return response;
  }

  async handle_by_address_query(
    address: string,
    requestBody: EndpointTypes[typeof route]['input'],
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    const response = await tx<ErrorShape | ProjectedNftRangeResponse>(pool, async dbTx => {
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

      const data = await projectedNftRangeByAddress({
        address: address,
        after: pageStartWithSlot?.tx_id || 0,
        until: until.tx_id,
        limit: requestBody.limit || PROJECTED_NFT_LIMIT.DEFAULT_PAGE_SIZE,
        dbTx,
      });

      return data.map(data => ({
        block: data.block,
        txId: data.tx_id,
        payload: (
          (data.payload as ({ [key: string]: string } & {
            actionSlot: number;
            actionOutputIndex: number;
          })[]) || []
        ).map(data => ({
          ownerAddress: data.ownerAddress,
          previousTxHash: data.previousUtxoTxHash,
          previousTxOutputIndex:
            data.previousTxOutputIndex != null ? parseInt(data.previousTxOutputIndex) : null,
          actionOutputIndex: data.actionOutputIndex,
          policyId: data.policyId,
          assetName: data.assetName,
          amount: data.amount,
          status: data.status as ProjectedNftStatus,
          plutusDatum: data.plutusDatum,
          actionSlot: data.actionSlot,
          forHowLong: data.forHowLong,
        })),
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
