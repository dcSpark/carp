import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { TransactionHistoryRequest } from '../models/TransactionHistory';
import type { TransactionHistoryResponse } from '../models/TransactionHistory';
import { countTxs } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';

@Route('transactions')
export class TransactionController extends Controller {
  @SuccessResponse(`${StatusCodes.OK}`, 'Created')
  @Post()
  public async txHistoryForAddresses(
    @Body()
    requestBody: TransactionHistoryRequest,
    @Res()
    errorResponse: TsoaResponse<404 /* TODO: change */, { reason: string }>
  ): Promise<TransactionHistoryResponse> {
    return await countTxs(
      // TODO: this is not what the real logic should be
      requestBody.addresses.map(addr => Buffer.from(addr, 'hex'))
    );
  }
}
