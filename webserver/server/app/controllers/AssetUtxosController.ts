import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { genErrorMessage, type ErrorShape, Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getAssetUtxos } from '../services/AssetUtxos';
import type { AssetUtxosResponse } from '../../../shared/models/AssetUtxos';
import type { IAssetUtxosResult } from '../models/asset/assetUtxos.queries';
import { bech32 } from 'bech32';
import { ASSET_UTXOS_LIMIT } from '../../../shared/constants';
import { Address } from '@dcspark/cardano-multiplatform-lib-nodejs';
import {
  adjustToSlotLimits,
  resolvePageStart,
  resolveUntilTransaction,
} from '../services/PaginationService';
import { slotBoundsPagination } from '../models/pagination/slotBoundsPagination.queries';
import { expectType } from 'tsd';

const route = Routes.assetUtxos;

@Route('asset/utxos')
export class AssetUtxosController extends Controller {
  /**
   * Returns utxo entries filtered either by cip 14 fingerprint or by policy id.
   *
   * This is useful to keep track of the utxo set of a particular asset.
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async assetUtxos(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    const assetsLength =
      (requestBody.fingerprints?.length || 0) + (requestBody.policyIds?.length || 0);
    if (assetsLength > ASSET_UTXOS_LIMIT.ASSETS) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AssetLimitExceeded, {
          limit: ASSET_UTXOS_LIMIT.ASSETS,
          found: assetsLength,
        })
      );
    }

    const response = await tx<ErrorShape | AssetUtxosResponse>(pool, async dbTx => {
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

      const data = await getAssetUtxos({
        after: pageStartWithSlot?.tx_id || 0,
        until: until.tx_id,
        limit: requestBody.limit || ASSET_UTXOS_LIMIT.DEFAULT_PAGE_SIZE,
        fingerprints: requestBody.fingerprints?.map(asset => {
          const decoded = bech32.decode(asset);
          const payload = bech32.fromWords(decoded.words);

          return Buffer.from(payload);
        }),
        policyIds: requestBody.policyIds?.map(policyId => Buffer.from(policyId, 'hex')),
        dbTx,
      });

      return data.map((data: IAssetUtxosResult) => {
        return {
          txId: data.tx as string,
          block: data.block,
          payload: (data.payload as { [key: string]: string | number }[]).map(x => {
            const address = Address.from_raw_bytes(
              Uint8Array.from(Buffer.from(x.addressRaw as string, 'hex'))
            );

            const paymentCred = address.payment_cred();
            const addressBytes = paymentCred?.to_cbor_bytes();

            return {
              utxo: {
                index: x.outputIndex,
                tx: x.outputTxHash,
              },
              paymentCred: Buffer.from(addressBytes as Uint8Array).toString('hex'),
              amount: x.amount ? x.amount : undefined,
              slot: x.slot,
              cip14Fingerprint: bech32.encode(
                'asset',
                bech32.toWords(Buffer.from(x.cip14Fingerprint as string, 'hex'))
              ),
              policyId: x.policyId,
              assetName: x.assetName,
            };
          }),
        } as AssetUtxosResponse[0];
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