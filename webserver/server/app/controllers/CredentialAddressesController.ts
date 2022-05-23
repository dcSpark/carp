import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { ADDRESS_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilTransaction } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import type {
  CredentialAddressRequest,
  CredentialAddressResponse,
} from '../../../shared/models/CredentialAddress';

const route = Routes.credentialAddress;

@Route('credential/address')
export class CredentialAddressesController extends Controller {
  /**
   * Ordered lexicographically (order is not maintained)
   *
   * Note: this endpoint only returns addresses that are in a block. Use another tool to see mempool information
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async addressesForCredential(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.credentials.length > ADDRESS_LIMIT.REQUEST) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AddressLimitExceeded, {
          limit: ADDRESS_LIMIT.REQUEST,
          found: requestBody.credentials.length,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const addresses = await tx<ErrorShape | [CredentialAddressRequest, CredentialAddressResponse]>(
      pool,
      async dbTx => {
        const [until] = await Promise.all([
          resolveUntilTransaction({
            block_hash: Buffer.from(requestBody.untilBlock, 'hex'),
            dbTx,
          }),
        ]);
        if (until == null) {
          return genErrorMessage(Errors.BlockHashNotFound, {
            untilBlock: requestBody.untilBlock,
          });
        }

        const commonRequest = {
          until,
          offset: requestBody.offset,
        };
        const result = await Promise.all([
          addressesForCredential({
            credentials: requestBody.credentials.map(addr => Buffer.from(addr, 'hex')),
            ...commonRequest,
          }),
        ]);
        return result;
      }
    );
    if ('code' in addresses) {
      expectType<Equals<typeof addresses, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.UNPROCESSABLE_ENTITY, addresses);
    }

    const result = [...addresses[0].addresses, ...addresses[1].addresses];
    result.sort();
    return {
      addresses: result,
    };
  }
}
