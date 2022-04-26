import { Body, Controller, TsoaResponse, Res, Post, Route, SuccessResponse } from 'tsoa';
import { historyForAddress } from '../services/TransactionHistoryService';
import { StatusCodes } from 'http-status-codes';
import type {
  TransactionHistoryRequest,
  TransactionHistoryResponse,
} from '../../../shared/models/TransactionHistory';
import assertNever from 'assert-never';

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
    return await historyForAddress(requestBody.addresses.map(addr => Buffer.from(addr, 'hex')));

    //   const verifiedBody = utils.validateHistoryReq(
    //     ADDRESS_REQUEST_LIMIT,
    //     ADDRESS_RESPONSE_LIMIT,
    //     req.body
    //   );
    //   switch (verifiedBody.kind) {
    //     case "ok": {
    //       const body = verifiedBody.value;
    //       const limit = body.limit || apiResponseLimit;
    //       const [referenceTx, referenceBlock] =
    //         (body.after && [body.after.tx, body.after.block]) || [];
    //       const referenceBestBlock = body.untilBlock;
    //       const untilBlockNum = await askBlockNumByHash(pool, referenceBestBlock);
    //       const afterBlockInfo = await askBlockNumByTxHash(pool, referenceTx);

    //       if (
    //         untilBlockNum.kind === "error" &&
    //         untilBlockNum.errMsg === utils.errMsgs.noValue
    //       ) {
    //         throw new Error("REFERENCE_BEST_BLOCK_MISMATCH");
    //       }
    //       if (
    //         afterBlockInfo.kind === "error" &&
    //         typeof referenceTx !== "undefined"
    //       ) {
    //         throw new Error("REFERENCE_TX_NOT_FOUND");
    //       }

    //       if (
    //         afterBlockInfo.kind === "ok" &&
    //         afterBlockInfo.value.block.hash !== referenceBlock
    //       ) {
    //         throw new Error("REFERENCE_BLOCK_MISMATCH");
    //       }

    //       // when things are running smoothly, we would never hit this case case
    //       if (untilBlockNum.kind !== "ok") {
    //         throw new Error(untilBlockNum.errMsg);
    //       }
    //       const afterInfo = getOrDefaultAfterParam(afterBlockInfo);

    //       const maybeTxs = await askTransactionHistory(
    //         pool,
    //         limit,
    //         body.addresses,
    //         afterInfo,
    //         untilBlockNum.value
    //       );
    //       switch (maybeTxs.kind) {
    //         case "ok": {
    //           const txs = mapTransactionFragsToResponse(maybeTxs.value);

    //           if (req.headers?.["flint-version"]) {
    //             const userFlintVersion = req.headers?.["flint-version"];

    //             // https://github.com/substack/semver-compare
    //             const flintSupportsApiVersion = semverCompare(
    //               userFlintVersion,
    //               FLINT_VERSION_WITH_API_VERSION_SUPPORT
    //             );
    //             // if userFlintVersion >=  FLINT_VERSION_WITH_API_VERSION_SUPPORT
    //             if (flintSupportsApiVersion >= 0) {
    //               res.send({ txs, version: TX_HISTORY_API_VERSION });
    //               return;
    //             }
    //           }
    //           res.send(txs);
    //           return;
    //         }
    //         case "error":
    //           throw new Error(maybeTxs.errMsg);
    //         default:
    //           return utils.assertNever(maybeTxs);
    //       }
    //     }
    //     case "error":
    //       throw new Error(verifiedBody.errMsg);
    //     default:
    //       return utils.assertNever(verifiedBody);
  }
}

// export const validateHistoryReq = (
//   addressRequestLimit: number,
//   apiResponseLimit: number,
//   data: any
// ): UtilEither<HistoryRequest> => {
//   if (!("addresses" in data))
//     return { kind: "error", errMsg: "body.addresses does not exist." };
//   if (!("untilBlock" in data))
//     return { kind: "error", errMsg: "body.untilBlock does not exist." };
//   if ("after" in data && !("tx" in data.after))
//     return {
//       kind: "error",
//       errMsg: "body.after exists but body.after.tx does not",
//     };
//   if ("after" in data && !("block" in data.after))
//     return {
//       kind: "error",
//       errMsg: "body.after exists but body.after.block does not",
//     };

//   const validatedAddresses = validateAddressesReq(
//     addressRequestLimit,
//     data.addresses
//   );
//   switch (validatedAddresses.kind) {
//     case "ok":
//       return { kind: "ok", value: data };
//     case "error":
//       return {
//         kind: "error",
//         errMsg: "body.addresses: " + validatedAddresses.errMsg,
//       };
//     default:
//       return assertNever(validatedAddresses);
//   }
// };

// /**
//  * This method validates addresses request body
//  * @param {Array[String]} addresses
//  */
//  export const validateAddressesReq = (
//   addressRequestLimit: number,
//   addresses: string[]
// ): UtilEither<string[]> => {
//   const errorMessage = `Addresses request length should be (0, ${addressRequestLimit}]`;
//   if (!addresses) {
//     return { kind: "error", errMsg: errorMessage };
//   } else if (addresses.length === 0 || addresses.length > addressRequestLimit) {
//     return { kind: "error", errMsg: errorMessage };
//   }
//   return { kind: "ok", value: addresses };
// };
