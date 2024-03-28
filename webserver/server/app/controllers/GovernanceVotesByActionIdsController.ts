import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { resolveUntilTransaction } from '../services/PaginationService';
import { expectType } from 'tsd';
import { governanceCredentialDidVote } from '../services/GovernanceCredentialVotesByActionIds';
import { GOVERNANCE_VOTES_BY_GOV_IDS_LIMIT } from '../../../shared/constants';

const route = Routes.governanceCredentialVotesByGovActionId;

@Route('governance/credential/votesByGovId')
export class GovernanceCredentialVotesByGovId extends Controller {
  /**
   * Gets votes cast for a set of governance action ids.
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async governanceCredentialDidVote(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.CONFLICT | StatusCodes.UNPROCESSABLE_ENTITY,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {

    if (requestBody.actionIds.length > GOVERNANCE_VOTES_BY_GOV_IDS_LIMIT.MAX_ACTION_IDS) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.GovActionIdsLimitExceeded, {
          limit: GOVERNANCE_VOTES_BY_GOV_IDS_LIMIT.MAX_ACTION_IDS,
          found: requestBody.actionIds.length,
        })
      );
    }

    let credential = Buffer.from(requestBody.credential, 'hex');

    const response = await tx<EndpointTypes[typeof route]['response'] | ErrorShape>(
      pool,
      async dbTx => {
        const until = await resolveUntilTransaction({
          block_hash: Buffer.from(requestBody.untilBlock, 'hex'),
          dbTx,
        });

        if (until == null) {
          return genErrorMessage(Errors.BlockHashNotFound, {
            untilBlock: requestBody.untilBlock,
          });
        }

        if(requestBody.actionIds.length === 0) {
          return [];
        }

        const data = await governanceCredentialDidVote({
          credential,
          govActionIds: requestBody.actionIds.map(actionId => Buffer.from(actionId, 'hex')),
          until: until.tx_id,
          dbTx,
        });

        return data.map(vote => ({
          actionId: vote.govActionId.toString('hex'),
          txId: vote.txId,
          payload: vote.vote.toString('hex'),
        }));
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
