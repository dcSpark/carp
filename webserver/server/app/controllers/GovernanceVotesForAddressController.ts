import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { governanceVotesForAddress } from '../services/GovernanceVotesForAddress';
import { resolvePageStart, resolveUntilTransaction } from '../services/PaginationService';
import { GOVERNANCE_VOTES_LIMIT } from '../../../shared/constants';
import { expectType } from 'tsd';
import { GovernanceVotesForAddressResponse } from '../../../shared/models/Governance';

const route = Routes.governanceVotesForAddress;

@Route('governance/votes/address')
export class GovernanceVotesForAddress extends Controller {
  /**
   * Returns the drep of the last delegation for this address.
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async governanceVotesForAddress(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    let credential = Buffer.from(requestBody.credential, 'hex');

    const response = await tx<EndpointTypes[typeof route]['response'] | ErrorShape>(
      pool,
      async dbTx => {
        const [until, pageStart] = await Promise.all([
          resolveUntilTransaction({ block_hash: Buffer.from(requestBody.untilBlock, 'hex'), dbTx }),
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

        if (requestBody.after && !pageStart) {
          return genErrorMessage(Errors.PageStartNotFound, {
            blockHash: requestBody.after.block,
            txHash: requestBody.after.tx,
          });
        }

        const data = await governanceVotesForAddress({
          credential,
          before: pageStart?.tx_id || Number.MAX_SAFE_INTEGER,
          until: until.tx_id,
          limit: requestBody.limit || GOVERNANCE_VOTES_LIMIT.DEFAULT_PAGE_SIZE,
          dbTx,
        });

        return data as GovernanceVotesForAddressResponse;
      }
    );

    if ('code' in response) {
      expectType<Equals<typeof response, ErrorShape>>(true);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(StatusCodes.CONFLICT, response);
    }

    return response;
  }
}
