import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { ASSET_LIMIT } from '../../../shared/constants';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { metadataNfts } from '../services/MetadataNft';
import type { NativeAsset } from '../../../shared/models/PolicyIdAssetMap';

const route = Routes.metadataNft;

@Route('metadata/nft')
export class MetadataNftController extends Controller {
  /**
   * Gets the CIP25 metadata for given <policy, asset_name> pairs
   *
   * Note: policy IDs and asset names MUST be in hex strings. Do not use UTF8 asset names.
   *
   * Note: This endpoint returns the NFT metadata as a CBOR object and NOT JSON.
   * You may expect a JSON object, but actually Cardano has no concept of on-chain JSON.
   * In fact, on-chain JSON is not even possible!
   * Instead, Cardano stores metadata as CBOR which can then be converted to JSON
   * The conversion of CBOR to JSON is project-dependent, and so Carp instead returns the raw cbor
   * It's up to you to choose how you want to do the JSON conversion.
   * The common case is handled inside the Carp client library.
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async metadataNft(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    const asset_pairs: NativeAsset[] = Object.entries(requestBody.assets).flatMap(
      ([policyId, assetNames]) => assetNames.map(assetName => [policyId, assetName] as NativeAsset)
    );
    if (asset_pairs.length > ASSET_LIMIT.REQUEST) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AssetLimitExceeded, {
          limit: ASSET_LIMIT.REQUEST,
          found: asset_pairs.length,
        })
      );
    }

    return await metadataNfts({
      dbTx: pool,
      pairs: asset_pairs,
    });
  }
}
