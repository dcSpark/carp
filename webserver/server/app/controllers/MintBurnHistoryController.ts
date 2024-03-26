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
import {
  getTransactionInputs,
  type ISqlMintBurnRangeResult,
} from '../models/asset/mintBurnHistory.queries';
import {
  adjustToSlotLimits,
  resolvePageStart,
  resolveUntilTransaction,
} from '../services/PaginationService';
import { slotBoundsPagination } from '../models/pagination/slotBoundsPagination.queries';
import { MINT_BURN_HISTORY_LIMIT } from '../../../shared/constants';
import { expectType } from 'tsd';
import { TransactionOutput } from '@dcspark/cardano-multiplatform-lib-nodejs';
import { BufferArray } from '../models/address/sqlAddressUsed.queries';

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

      let assets;
      if (requestBody.policyIds !== undefined && requestBody.policyIds.length > 0) {
        assets = await mintBurnRangeByPolicyIds({
          after: pageStartWithSlot?.tx_id || 0,
          until: until.tx_id,
          limit: requestBody.limit || MINT_BURN_HISTORY_LIMIT.DEFAULT_PAGE_SIZE,
          policyIds: requestBody.policyIds,
          dbTx,
        });
      } else {
        assets = await mintBurnRange({
          after: pageStartWithSlot?.tx_id || 0,
          until: until.tx_id,
          limit: requestBody.limit || MINT_BURN_HISTORY_LIMIT.DEFAULT_PAGE_SIZE,
          dbTx,
        });
      }

      const txs = assets.map(entry => entry.tx_db_id);

      const inputs =
        txs.length > 0
          ? Object.fromEntries(
              (await getTransactionInputs.run({ tx_ids: txs }, dbTx)).map(tx => [
                tx.tx_id,
                tx.input_payloads,
              ])
            )
          : {};

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

        const f = (payloads: BufferArray) => {
          const inputAddresses: {
            [address: string]: { policyId: string; assetName: string; amount: string }[];
          } = {};

          for (const rawOutput of payloads) {
            const output = TransactionOutput.from_cbor_bytes(rawOutput);

            const inputAddress = output.address().payment_cred()?.to_cbor_hex();

            if (!inputAddress) continue;

            const ma = output.amount().multi_asset();

            const policyIdsInInput = ma.keys();

            for (let i = 0; i < policyIdsInInput.len(); i++) {
              const policyId = policyIdsInInput.get(i);

              const hexPolicyId = policyId.to_hex();

              const assetsInOutput = ma.get_assets(policyId);

              const assetNames = assetsInOutput?.keys();

              if (!assetNames) continue;

              for (let j = 0; j < assetNames.len(); j++) {
                const assetName = assetNames.get(j);
                const hexAssetName = Buffer.from(assetName.get()).toString('hex');

                if (assets[hexPolicyId] && assets[hexPolicyId][hexAssetName]) {
                  if (!inputAddresses[inputAddress]) {
                    inputAddresses[inputAddress] = [];
                  }

                  inputAddresses[inputAddress].push({
                    policyId: hexPolicyId,
                    assetName: hexAssetName,
                    amount: assetsInOutput?.get(assetName)?.toString()!,
                  });
                }
              }
            }
          }

          return inputAddresses;
        };

        const inputAddresses = f(inputs[entry.tx_db_id] || []);
        const outputAddresses = f(entry.output_payloads || []);

        return {
          assets: assets,
          actionSlot: entry.action_slot,
          metadata: entry.action_tx_metadata,
          txId: entry.tx,
          block: entry.block,
          inputAddresses,
          outputAddresses,
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
