import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { CREDENTIAL_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolveUntilTransaction } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import type { CredentialAddressResponse } from '../../../shared/models/CredentialAddress';
import { addressesForCredential } from '../services/CredentialAddressesService';
import { getAsCredentialHex, getAsExactAddressHex } from '../models/utils';

const route = Routes.credentialAddress;

@Route('credential/address')
export class CredentialAddressesController extends Controller {
  /**
   * Ordered by the first time the address was seen on-chain
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

    let after: undefined | { address: Buffer } = undefined;
    if (requestBody.after != null) {
      const asHex = getAsExactAddressHex(requestBody.after);
      if (asHex == null) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        return errorResponse(
          StatusCodes.UNPROCESSABLE_ENTITY,
          genErrorMessage(Errors.IncorrectAddressFormat, {
            addresses: [requestBody.after],
          })
        );
      }
      after = { address: Buffer.from(asHex, 'hex') };
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
      const [until] = await Promise.all([
        resolveUntilTransaction({
          block_hash: Buffer.from(requestBody.untilBlock, 'hex'),
          dbTx,
        }),
      ]);
      if (until == null) {
        return genErrorMessage(Errors.BlockHashNotFound, {
          block: requestBody.untilBlock,
        });
      }

      const result = await addressesForCredential({
        after,
        credentials: requestBody.credentials.map(addr => Buffer.from(addr, 'hex')),
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
