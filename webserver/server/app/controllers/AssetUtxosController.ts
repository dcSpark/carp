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

const route = Routes.assetUtxos;

@Route('asset/utxos')
export class AssetUtxosController extends Controller {
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
    if (requestBody.assets.length > ASSET_UTXOS_LIMIT.ASSETS) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AssetLimitExceeded, {
          limit: ASSET_UTXOS_LIMIT.ASSETS,
          found: requestBody.assets.length,
        })
      );
    }

    const response = await tx<AssetUtxosResponse>(pool, async dbTx => {
      const data = await getAssetUtxos({
        range: requestBody.range,
        assets: requestBody.assets.map(asset => {
          const decoded = bech32.decode(asset);
          const payload = bech32.fromWords(decoded.words);

          return Buffer.from(payload);
        }),
        dbTx,
      });

      return data.map((data: IAssetUtxosResult): AssetUtxosResponse[0] => {
        const address = Address.from_bytes(Uint8Array.from(data.address_raw));

        const paymentCred = address.payment_cred();
        const addressBytes = paymentCred?.to_bytes();

        address.free();
        paymentCred?.free();

        return {
          txId: data.tx_hash as string,
          utxo: {
            index: data.output_index,
            tx: data.output_tx_hash as string,
          },
          paymentCred: Buffer.from(addressBytes as Uint8Array).toString('hex'),
          amount: data.amount ? data.amount : undefined,
          slot: data.slot,
          cip14Fingerprint: bech32.encode('asset', bech32.toWords(data.cip14_fingerprint)),
        };
      });
    });

    return response;
  }
}
