import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { historyForAddresses, historyForCredentials } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';
import {
  TransactionHistoryResponse,
  RelationFilterType,
} from '../../../shared/models/TransactionHistory';
import { bech32 } from 'bech32';
import {
  Address,
  ByronAddress,
  Ed25519KeyHash,
  ScriptHash,
  StakeCredential,
} from '@dcspark/cardano-multiplatform-lib-nodejs';
import Cip5 from '@dcspark/cip5-js';
import { ADDRESS_REQUEST_LIMIT, ADDRESS_RESPONSE_LIMIT } from '../../../shared/constants';
import { ParsedAddressTypes } from '../models/ParsedAddressTypes';
import tx from 'pg-tx';
import pool from '../services/PgPoolSingleton';
import { resolvePageStart, resolveUntilBlock } from '../services/PaginationService';
import { ErrorShape, genErrorMessage } from '../../../shared/errors';
import { Errors } from '../../../shared/errors';
import { expectType } from 'tsd';
import { EndpointTypes, Routes } from '../../../shared/routes';
import sortBy from 'lodash/sortBy';

const route = Routes.txsForAddresses;

@Route('txsForAddresses')
export class TransactionController extends Controller {
  /**
   * Ordered by `<block.height, transaction.tx_index>`
   * Note: this endpoint only returns txs that are in a block. Use another tool to see mempool for txs not in a block
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async txsForAddresses(
    @Body()
    requestBody: EndpointTypes[typeof route]['input'],
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<EndpointTypes[typeof route]['response']> {
    if (requestBody.addresses.length > ADDRESS_REQUEST_LIMIT) {
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
      return errorResponse(
        StatusCodes.BAD_REQUEST,
        genErrorMessage(Errors.IncorrectAddressFormat, {
          addresses: addressTypes.invalid,
        })
      );
    }

    // note: we use a SQL transaction to make sure the pagination check works properly
    // otherwise, a rollback could happen between getting the pagination info and the history query
    const cardanoTxs = await tx<
      ErrorShape | [TransactionHistoryResponse, TransactionHistoryResponse]
    >(pool, async dbTx => {
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
        limit: requestBody.limit ?? ADDRESS_RESPONSE_LIMIT,
        until,
        dbTx,
      };
      const result = await Promise.all([
        historyForCredentials({
          stakeCredentials: addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex')),
          relationFilter: requestBody.relationFilter ?? RelationFilterType.NO_FILTER,
          ...commonRequest,
        }),
        historyForAddresses({
          addresses: [
            ...addressTypes.exactAddress.map(addr => Buffer.from(addr, 'hex')),
            ...addressTypes.exactLegacyAddress.map(addr => Buffer.from(addr, 'hex')),
          ],
          ...commonRequest,
        }),
      ]);
      return result;
    });
    if ('code' in cardanoTxs) {
      expectType<Equals<typeof cardanoTxs, ErrorShape>>(true);
      return errorResponse(StatusCodes.PRECONDITION_REQUIRED, cardanoTxs);
    }

    const mergedTxs = sortBy(
      [...cardanoTxs[0].transactions, ...cardanoTxs[1].transactions],
      [tx => tx.block.height, tx => tx.block.tx_ordinal]
    );

    return {
      transactions:
        mergedTxs.length > ADDRESS_RESPONSE_LIMIT
          ? mergedTxs.slice(0, ADDRESS_RESPONSE_LIMIT)
          : mergedTxs,
    };
  }
}

const credentialLength = 32 * 2; // 32 bytes = 64 hex letters

export const getAddressTypes = (addresses: string[]): ParsedAddressTypes => {
  const result: ParsedAddressTypes = {
    credentialHex: [],
    exactAddress: [],
    exactLegacyAddress: [],
    invalid: [],
  };
  const isCredentialHex = (address: string) =>
    new RegExp(`^[0-9a-fA-F]{${credentialLength}}$`).test(address);
  for (const address of addresses) {
    if (isCredentialHex(address)) {
      result.credentialHex.push(address);
      continue;
    }
    try {
      const bech32Info = bech32.decode(address, 1000);
      switch (bech32Info.prefix) {
        case Cip5.miscellaneous.addr:
        case Cip5.miscellaneous.addr_test:
          const payload = bech32.fromWords(bech32Info.words);
          result.exactAddress.push(Buffer.from(payload).toString('hex'));
          continue;
        case Cip5.miscellaneous.stake:
        case Cip5.miscellaneous.stake_test: {
          const addr = Address.from_bech32(address);
          const rewardAddr = addr.as_reward();
          if (rewardAddr == null) {
            result.invalid.push(address);
            addr.free();
          } else {
            const cred = rewardAddr.payment_cred();
            result.credentialHex.push(Buffer.from(cred.to_bytes()).toString('hex'));
            addr.free();
            cred.free();
          }
          continue;
        }
        case Cip5.hashes.addr_vkh:
        case Cip5.hashes.policy_vkh:
        case Cip5.hashes.stake_vkh:
        case Cip5.hashes.stake_shared_vkh:
        case Cip5.hashes.addr_shared_vkh: {
          const payload = bech32.fromWords(bech32Info.words);
          const keyHash = Ed25519KeyHash.from_bytes(Buffer.from(payload));
          const stakeCred = StakeCredential.from_keyhash(keyHash);
          result.credentialHex.push(Buffer.from(stakeCred.to_bytes()).toString('hex'));
          keyHash.free();
          stakeCred.free();
          continue;
        }
        case Cip5.hashes.script: {
          const payload = bech32.fromWords(bech32Info.words);
          const keyHash = ScriptHash.from_bytes(Buffer.from(payload));
          const stakeCred = StakeCredential.from_scripthash(keyHash);
          result.credentialHex.push(Buffer.from(stakeCred.to_bytes()).toString('hex'));
          keyHash.free();
          stakeCred.free();
          continue;
        }
      }
    } catch (_e) {}
    if (ByronAddress.is_valid(address)) {
      const byronAddr = ByronAddress.from_base58(address);
      result.exactLegacyAddress.push(Buffer.from(byronAddr.to_bytes()).toString('hex'));
      byronAddr.free();
      continue;
    }
    result.invalid.push(address);
  }

  return result;
};
