import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { UTXO_LIMIT } from '../../../shared/constants';
import pool from '../services/PgPoolSingleton';
import type { ErrorShape } from '../../../shared/errors';
import { genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import type { EndpointTypes } from '../../../shared/routes';
import { Routes } from '../../../shared/routes';
import { outputsForTransaction } from '../services/TransactionOutput';
import { isTxHash } from '../models/utils';

const route = Routes.transactionOutput;

@Route('transaction/output')
export class TransactionOutputController extends Controller {
  /**
   * Get the outputs for given `<tx hash, output index>` pairs
   *
   * Note: this endpoint only returns txs that are in a block. Use another tool to see mempool for txs not in a block
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async transactionOutput(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.utxoPointers.length > UTXO_LIMIT.REQUEST) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.UtxoLimitExceeded, {
          limit: UTXO_LIMIT.REQUEST,
          found: requestBody.utxoPointers.length,
        })
      );
    }

    const invalidHashes: string[] = [];
    requestBody.utxoPointers
      .filter(pointer => !isTxHash(pointer.txHash))
      .forEach(invalid => invalidHashes.push(invalid.txHash));
    if (invalidHashes.length > 0) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return errorResponse(
        StatusCodes.PRECONDITION_REQUIRED,
        genErrorMessage(Errors.IncorrectTxHashFormat, {
          txHash: invalidHashes,
        })
      );
    }
    const { utxos } = await outputsForTransaction({
      dbTx: pool,
      utxoPointers: requestBody.utxoPointers,
    });
    return {
      utxos,
    };
  }
}
