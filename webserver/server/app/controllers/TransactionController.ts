import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { historyForAddress } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from '../../../shared/models/TransactionHistory';
import assertNever from 'assert-never';
import { bech32 } from 'bech32';
import {
  Address,
  ByronAddress,
  BaseAddress,
  PointerAddress,
  EnterpriseAddress,
  RewardAddress,
  Transaction,
} from '@dcspark/cardano-multiplatform-lib-nodejs';
import Cip5 from '@dcspark/cip5-js';

@Route('txsForAddress')
export class TransactionController extends Controller {
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
    // TODO: support other types
    return await historyForAddress(
      addressTypes.credentialHex.map(addr => Buffer.from(addr, 'hex'))
    );

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

type ParsedAddressTypes = {
  credentialHex: string[];
  exactAddress: string[];
  exactLegacyAddress: string[];
  invalid: string[];
};

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
          // TODO: convert to credential
          const payload = bech32.fromWords(bech32Info.words);
          result.credentialHex.push(Buffer.from(payload).toString('hex'));
          continue;
        }
        case Cip5.hashes.script: {
          // TODO: convert to credential
          const payload = bech32.fromWords(bech32Info.words);
          result.credentialHex.push(Buffer.from(payload).toString('hex'));
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
