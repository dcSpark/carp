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

@Route('txsForAddress')
export class TransactionController extends Controller {
  /**
   * Ordered by <block.height, transaction.tx_index>
   * Transactions that are in a block appear before txs that aren't
   * Two txs that both aren't in a block are sorted by their position in the mempool
   */
  @SuccessResponse(`${StatusCodes.OK}`, 'Created')
  @Post()
  public async txsForAddress(
    @Body()
    requestBody: TransactionHistoryRequest,
    @Res()
    errorResponse: TsoaResponse<StatusCodes.BAD_REQUEST, { reason: string }>
  ): Promise<TransactionHistoryResponse> {
    if (requestBody.addresses.length > ADDRESS_REQUEST_LIMIT) {
      throw new Error(); // TODO: proper error
    }
    const addressTypes = getAddressTypes(requestBody.addresses);
    if (addressTypes.invalid.length > 0) {
    }
    const txs = await Promise.all([
      historyForCredentials(addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex'))),
      historyForAddresses([
        ...addressTypes.exactAddress.map(addr => Buffer.from(addr, 'hex')),
        ...addressTypes.exactLegacyAddress.map(addr => Buffer.from(addr, 'hex')),
      ]),
    ]);

    const mergedTxs = [...txs[0].transactions, ...txs[1].transactions];
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

    // switch (verifiedBody.kind) {
    //   case "ok": {
    //     const body = verifiedBody.value;
    //     const limit = body.limit || apiResponseLimit;
    //     const [referenceTx, referenceBlock] =
    //       (body.after && [body.after.tx, body.after.block]) || [];
    //     const referenceBestBlock = body.untilBlock;
    //     const untilBlockNum = await askBlockNumByHash(pool, referenceBestBlock);
    //     const afterBlockInfo = await askBlockNumByTxHash(pool, referenceTx);

    //     if (
    //       untilBlockNum.kind === "error" &&
    //       untilBlockNum.errMsg === utils.errMsgs.noValue
    //     ) {
    //       throw new Error("REFERENCE_BEST_BLOCK_MISMATCH");
    //     }
    //     if (
    //       afterBlockInfo.kind === "error" &&
    //       typeof referenceTx !== "undefined"
    //     ) {
    //       throw new Error("REFERENCE_TX_NOT_FOUND");
    //     }

    //     if (
    //       afterBlockInfo.kind === "ok" &&
    //       afterBlockInfo.value.block.hash !== referenceBlock
    //     ) {
    //       throw new Error("REFERENCE_BLOCK_MISMATCH");
    //     }

    //     // when things are running smoothly, we would never hit this case case
    //     if (untilBlockNum.kind !== "ok") {
    //       throw new Error(untilBlockNum.errMsg);
    //     }
    //     const afterInfo = getOrDefaultAfterParam(afterBlockInfo);

    //     const maybeTxs = await askTransactionHistory(
    //       pool,
    //       limit,
    //       body.addresses,
    //       afterInfo,
    //       untilBlockNum.value
    //     );
    //     switch (maybeTxs.kind) {
    //       case "ok": {
    //         const txs = mapTransactionFragsToResponse(maybeTxs.value);

    //         if (req.headers?.["flint-version"]) {
    //           const userFlintVersion = req.headers?.["flint-version"];

    //           // https://github.com/substack/semver-compare
    //           const flintSupportsApiVersion = semverCompare(
    //             userFlintVersion,
    //             FLINT_VERSION_WITH_API_VERSION_SUPPORT
    //           );
    //           // if userFlintVersion >=  FLINT_VERSION_WITH_API_VERSION_SUPPORT
    //           if (flintSupportsApiVersion >= 0) {
    //             res.send({ txs, version: TX_HISTORY_API_VERSION });
    //             return;
    //           }
    //         }
    //         res.send(txs);
    //         return;
    //       }
    //       case "error":
    //         throw new Error(maybeTxs.errMsg);
    //       default:
    //         return utils.assertNever(maybeTxs);
    //     }
    //   }
    //   case "error":
    //     throw new Error(verifiedBody.errMsg);
    //   default:
    //     return utils.assertNever(verifiedBody);
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
