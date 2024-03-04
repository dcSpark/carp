import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';

import type { ErrorShape } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { mintBurnRange, mintBurnRangeByPolicyIds } from '../services/MintBurnHistoryService';
import type { MintBurnSingleResponse } from '../../../shared/models/MintBurn';
import type { PolicyId } from '../../../shared/models/PolicyIdAssetMap';
import type {
  ISqlMintBurnRangeResult,
  ISqlMintBurnRangeByPolicyIdsResult,
} from '../models/asset/mintBurnHistory.queries';

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
  public async projectedNftRange(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.policyIds !== undefined && requestBody.policyIds.length > 0) {
      return await this.handle_by_policy_ids_query(requestBody.policyIds, requestBody);
    } else {
      return await this.handle_general_query(requestBody);
    }
  }

  async handle_general_query(
    requestBody: EndpointTypes[typeof route]['input']
  ): Promise<EndpointTypes[typeof route]['response']> {
    const assets = await tx<ISqlMintBurnRangeResult[]>(pool, async dbTx => {
      const data = await mintBurnRange({
        range: requestBody.range,
        dbTx,
      });

      return data;
    });

    let mintRangeResponse: MintBurnSingleResponse = {
      actionTxId: '',
      actionBlockId: '',
      metadata: null,
      actionSlot: 0,
      assets: {},
    };

    const result: MintBurnSingleResponse[] = [];

    for (const entry of assets) {
      const policyId = entry.policy_id !== null ? entry.policy_id.toString() : '';
      const assetName = entry.asset_name !== null ? entry.asset_name.toString() : '';
      const actionTxId = entry.action_tx_id !== null ? entry.action_tx_id.toString() : '';
      const actionBlockId = entry.action_block_id !== null ? entry.action_block_id.toString() : '';

      if (mintRangeResponse.actionTxId != actionTxId) {
        if (mintRangeResponse.actionTxId.length > 0) {
          result.push(mintRangeResponse);
        }

        mintRangeResponse = {
          actionSlot: entry.action_slot,
          actionTxId: actionTxId,
          actionBlockId: actionBlockId,
          metadata: entry.action_tx_metadata,
          assets: {},
        };
      }

      const for_policy = mintRangeResponse.assets[policyId] ?? {};

      for_policy[assetName] = entry.amount;
      mintRangeResponse.assets[policyId] = for_policy;
    }

    if (mintRangeResponse.actionTxId.length > 0) {
      result.push(mintRangeResponse);
    }

    return result;
  }

  async handle_by_policy_ids_query(
    policyIds: PolicyId[],
    requestBody: EndpointTypes[typeof route]['input']
  ): Promise<EndpointTypes[typeof route]['response']> {
    const assets = await tx<ISqlMintBurnRangeByPolicyIdsResult[]>(pool, async dbTx => {
      const data = await mintBurnRangeByPolicyIds({
        range: requestBody.range,
        policyIds: policyIds,
        dbTx,
      });

      return data;
    });

    let mintRangeResponse: MintBurnSingleResponse = {
      actionTxId: '',
      actionBlockId: '',
      metadata: null,
      actionSlot: 0,
      assets: {},
    };

    const result: MintBurnSingleResponse[] = [];

    for (const entry of assets) {
      const policyId = entry.policy_id !== null ? entry.policy_id.toString() : '';
      const assetName = entry.asset_name !== null ? entry.asset_name.toString() : '';
      const actionTxId = entry.action_tx_id !== null ? entry.action_tx_id.toString() : '';
      const actionBlockId = entry.action_block_id !== null ? entry.action_block_id.toString() : '';

      if (mintRangeResponse.actionTxId != actionTxId) {
        if (mintRangeResponse.actionTxId.length > 0) {
          result.push(mintRangeResponse);
        }

        mintRangeResponse = {
          actionSlot: entry.action_slot,
          actionTxId: actionTxId,
          actionBlockId: actionBlockId,
          metadata: entry.action_tx_metadata,
          assets: {},
        };
      }

      const for_policy = mintRangeResponse.assets[policyId] ?? {};

      for_policy[assetName] = entry.amount;
      mintRangeResponse.assets[policyId] = for_policy;
    }

    if (mintRangeResponse.actionTxId.length > 0) {
      result.push(mintRangeResponse);
    }

    return result;
  }
}
