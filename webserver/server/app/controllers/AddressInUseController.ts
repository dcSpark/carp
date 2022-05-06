import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { ADDRESS_REQUEST_LIMIT } from '../../../shared/constants';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilBlock } from '../services/PaginationService';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { getAddressTypes } from '../models/utils';
import type { AddressUsedResponse } from '../../../shared/models/AddressUsed';
import { addressUsed, credentialUsed } from '../services/AddressUsedService';
import { RelationFilterType } from '../../../shared/models/common';

const route = Routes.addressUsed;

@Route('address/used')
export class AddressInUseController extends Controller {
  /**
   * Ordered lexicographically (order is not maintained)
   * Note: this endpoint only returns addresses that are in a block. Use another tool to see mempool information
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async addressUsed(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.addresses.length > ADDRESS_REQUEST_LIMIT) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.AddressLimitExceeded, {
          limit: ADDRESS_REQUEST_LIMIT,
          found: requestBody.addresses.length,
        })
      );
    }
    const addressTypes = getAddressTypes(requestBody.addresses);
    if (addressTypes.invalid.length > 0) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.IncorrectAddressFormat, {
          addresses: addressTypes.invalid,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const addresses = await tx<ErrorShape | [AddressUsedResponse, AddressUsedResponse]>(
      pool,
      async dbTx => {
        const [until, pageStart] = await Promise.all([
          resolveUntilBlock({
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
          return genErrorMessage(Errors.UntilBlockNotFound, {
            untilBlock: requestBody.untilBlock,
          });
        }
        if (requestBody.after != null && pageStart == null) {
          return genErrorMessage(Errors.PageStartNotFound, {
            blockHash: requestBody.after.block,
            txHash: requestBody.after.tx,
          });
        }

        const commonRequest = {
          after: pageStart,
          until,
          dbTx,
          reverseMap: addressTypes.reverseMap,
        };
        const result = await Promise.all([
          credentialUsed({
            stakeCredentials: addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex')),
            relationFilter: requestBody.relationFilter ?? RelationFilterType.NO_FILTER,
            ...commonRequest,
          }),
          addressUsed({
            addresses: [
              ...addressTypes.exactAddress.map(addr => Buffer.from(addr, 'hex')),
              ...addressTypes.exactLegacyAddress.map(addr => Buffer.from(addr, 'hex')),
            ],
            ...commonRequest,
          }),
        ]);
        return result;
      }
    );
    console.log('-------------');
    console.log(JSON.stringify(addresses));
    if ('code' in addresses) {
      expectType<Equals<typeof addresses, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.PRECONDITION_REQUIRED, addresses);
    }

    const result = [...addresses[0].addresses, ...addresses[1].addresses];
    result.sort();
    console.log(result);
    return {
      addresses: result,
    };
  }
}
