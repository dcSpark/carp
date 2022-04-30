import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { historyForAddresses, historyForCredentials } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
  MempoolTx,
} from '../../../shared/models/TransactionHistory';
import { bech32 } from 'bech32';
import {
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
import { ErrorShape, genErrorMessage } from '../models/errors';
import { Errors } from '../models/errors';
import { expectType } from 'tsd';

@Route('txsForAddress')
export class TransactionController extends Controller {
  /**
   * Ordered by <block.height, transaction.tx_index>
   * Transactions that are in a block appear before txs that aren't
   * Two txs that both aren't in a block are sorted by their position in the mempool
   *
   * Addresses can be in the following form:
   * - Stake credential hex
   * - Bech32 (addr1, addr_vkh, etc.)
   * - Legacy Byron format (Ae2, Dd, etc.)
   */
  @SuccessResponse(`${StatusCodes.OK}`)
  @Post()
  public async txsForAddress(
    @Body()
    requestBody: TransactionHistoryRequest,
    @Res()
    errorResponse: TsoaResponse<
      StatusCodes.BAD_REQUEST | StatusCodes.PRECONDITION_REQUIRED,
      ErrorShape
    >
  ): Promise<TransactionHistoryResponse> {
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
        limit: ADDRESS_RESPONSE_LIMIT,
        until,
        dbTx,
      };
      const result = await Promise.all([
        historyForCredentials({
          stakeCredentials: addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex')),
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

    const mergedTxs = [...cardanoTxs[0].transactions, ...cardanoTxs[1].transactions];
    mergedTxs.sort((a, b) => {
      // return any tx in a block before txs not in a block
      if ('mempool' in a) {
        if ('mempool' in b) {
          // if neither are in a block, we just make sure the order is consistent
          return a.mempool.positionInMempool - b.mempool.positionInMempool;
        }
        return -1;
      }
      if ('mempool' in b) return 1;
      if (a.block.height === b.block.height) {
        return a.block.tx_ordinal - b.block.tx_ordinal;
      }
      return a.block.height - b.block.height;
    });

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
        case Cip5.miscellaneous.stake:
        case Cip5.miscellaneous.stake_test: {
          result.exactAddress.push(address);
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
      result.exactLegacyAddress.push(address);
      continue;
    }
    result.invalid.push(address);
  }

  return result;
};
