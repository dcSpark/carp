import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { CREDENTIAL_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilTransaction } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import type { CredentialAddressResponse } from '../../../shared/models/CredentialAddress';
import { addressesForCredential } from '../services/CredentialAddressesService';
import { getAsCredentialHex } from '../models/utils';

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
    if (requestBody.credentials.length > CREDENTIAL_LIMIT.REQUEST) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.CredentialLimitExceeded, {
          limit: CREDENTIAL_LIMIT.REQUEST,
          found: requestBody.credentials.length,
        })
      );
    }

    const filteredAddresses = {
      hex: [] as string[],
      invalid: [] as string[],
    };
    for (const cred of requestBody.credentials) {
      const asCredHex = getAsCredentialHex(cred);
      if (asCredHex == null) {
        filteredAddresses.invalid.push(cred);
      } else {
        filteredAddresses.hex.push(cred);
      }
    }
    if (filteredAddresses.invalid.length > 0) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.UNPROCESSABLE_ENTITY,
        genErrorMessage(Errors.IncorrectAddressFormat, {
          addresses: filteredAddresses.invalid,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const addresses = await tx<ErrorShape | CredentialAddressResponse>(pool, async dbTx => {
      const [until, pageStart] = await Promise.all([
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

      const result = await addressesForCredential({
        credentials: requestBody.credentials.map(addr => Buffer.from(addr, 'hex')),
        after: pageStart,
        until,
        dbTx,
      });
      return result;
    });
    if ('code' in addresses) {
      expectType<Equals<typeof addresses, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.UNPROCESSABLE_ENTITY, addresses);
    }

    return addresses;
  }
}
