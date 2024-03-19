import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';

import { Errors, genErrorMessage, type ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { mintBurnRange, mintBurnRangeByPolicyIds } from '../services/MintBurnHistoryService';
import type { MintBurnSingleResponse } from '../../../shared/models/MintBurn';
import type { PolicyId } from '../../../shared/models/PolicyIdAssetMap';
import type {
  ISqlMintBurnRangeResult,
} from '../models/asset/mintBurnHistory.queries';
import {
  adjustToSlotLimits,
  resolvePageStart,
  resolveUntilTransaction,
} from '../services/PaginationService';
import { slotBoundsPagination } from '../models/pagination/slotBoundsPagination.queries';
import { MINT_BURN_HISTORY_LIMIT } from '../../../shared/constants';
import { expectType } from 'tsd';

const route = Routes.mintBurnHistory;

@Route('asset/mint-burn-history')
export class MintRangeController extends Controller {
  /**
   * Gets mint and burn events in the provided slot range, optionally filtering
   * by policyId(s). A burn event is a mint with a negative value.
   *
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async mintBurnHistory(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const response = await tx<ErrorShape | MintBurnSingleResponse[]>(pool, async dbTx => {
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

      const assets = await tx<ISqlMintBurnRangeResult[]>(pool, async dbTx => {
        if (requestBody.policyIds !== undefined && requestBody.policyIds.length > 0) {
          const data = await mintBurnRangeByPolicyIds({
            after: pageStartWithSlot?.tx_id || 0,
            until: until.tx_id,
            limit: requestBody.limit || MINT_BURN_HISTORY_LIMIT.DEFAULT_PAGE_SIZE,
            policyIds: requestBody.policyIds,
            dbTx,
          });

          return data;
        } else {
          const data = await mintBurnRange({
            after: pageStartWithSlot?.tx_id || 0,
            until: until.tx_id,
            limit: requestBody.limit || MINT_BURN_HISTORY_LIMIT.DEFAULT_PAGE_SIZE,
            dbTx,
          });

          return data;
        }
      });

      return assets.map(entry => {
        const assets: { [policyId: PolicyId]: { [assetName: string]: string } } = {};

        for (const pair of entry.payload as {
          policyId: string;
          assetName: string;
          amount: string;
        }[]) {
          if (!assets[pair.policyId]) {
            assets[pair.policyId] = { [pair.assetName]: pair.amount };
          } else {
            assets[pair.policyId][pair.assetName] = pair.amount;
          }
        }

        return {
          assets: assets,
          actionSlot: entry.action_slot,
          metadata: entry.action_tx_metadata,
          txId: entry.tx,
          block: entry.block,
        };
      });
    });

    if ('code' in response) {
      expectType<Equals<typeof response, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, response);
    }

    return response;
  }
}
