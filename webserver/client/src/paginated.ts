import { query } from "./template";
import type { EndpointTypes } from "../../shared/routes";
import { Routes } from "../../shared/routes";
import type { Pagination } from "../../shared/models/common";
import type { TxAndBlockInfo } from "../../shared/models/TransactionHistory";
import type {
  NativeAsset,
  PolicyIdAssetMapType,
} from "../../shared/models/PolicyIdAssetMap";
import chunk from "lodash/chunk";
import { ASSET_LIMIT } from "../../shared/constants";
import merge from "lodash/merge";
import type cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import type { ProjectedNftRangeResponse } from "../../shared/models/ProjectedNftRange";

/**
 * If you don't mind using axios,
 * you can use the paginated endpoints provided by the client
 * However this endpoint allows you to pass in your own querying library
 */
export async function paginateQuery<T extends Pagination, Response>(
  initialRequest: T,
  query: (request: T) => Promise<Response[]>,
  pageFromResponse: (resp: undefined | Response) => Pagination["after"]
): Promise<Response[]> {
  let nextRequest: T = initialRequest;
  const result: Response[] = [];
  let currentPage: Response[] = [];
  do {
    currentPage = await query(nextRequest);
    result.push(...currentPage);

    nextRequest = {
      ...nextRequest,
      after: pageFromResponse(currentPage[currentPage.length - 1]),
    };
  // TODO: This could be more efficient if we know the max page size for each query
  } while (currentPage.length !== 0);

  return result;
}

export async function paginatedTransactionHistory(
  urlBase: string,
  initialRequest: Omit<
    EndpointTypes[Routes.transactionHistory]["input"],
    "after"
  >
): Promise<EndpointTypes[Routes.transactionHistory]["response"]> {
  const result = await paginateQuery<
    EndpointTypes[Routes.transactionHistory]["input"],
    TxAndBlockInfo
  >(
    initialRequest,
    async (request) =>
      (
        await query(urlBase, Routes.transactionHistory, request)
      ).transactions,
    (resp) =>
      resp != null
        ? {
            block: resp.block.hash,
            tx: resp.transaction.hash,
          }
        : undefined
  );
  return { transactions: result };
}

export async function paginatedProjectedNft(
  urlBase: string,
  initialRequest: Omit<
    EndpointTypes[Routes.projectedNftEventsRange]["input"],
    "after"
  >
): Promise<EndpointTypes[Routes.projectedNftEventsRange]["response"]> {
  const result = await paginateQuery<
    EndpointTypes[Routes.projectedNftEventsRange]["input"],
    ProjectedNftRangeResponse[number]
  >(
    initialRequest,
    async (request) =>
      (
        await query(urlBase, Routes.projectedNftEventsRange, request)
      ),
    (resp) =>
      resp != null
        ? {
            block: resp.block,
            tx: resp.txId,
          }
        : undefined
  );
  return result;
}

function pairsToAssetMap(pairs: NativeAsset[]): PolicyIdAssetMapType {
  const result: PolicyIdAssetMapType["assets"] = {};
  for (const [policyId, assetName] of pairs) {
    const for_policy = result[policyId] ?? [];

    for_policy.push(assetName);
    // if this was a newly added policy
    if (for_policy.length === 1) {
      result[policyId] = for_policy;
    }
  }

  return { assets: result };
}
export async function paginatedMetadataNft(
  urlBase: string,
  request: EndpointTypes[Routes.metadataNft]["input"]
): Promise<EndpointTypes[Routes.metadataNft]["response"]> {
  const pairs: NativeAsset[] = [];
  for (const [policyId, assets] of Object.entries(request.assets)) {
    for (const asset of assets) {
      pairs.push([policyId, asset]);
    }
  }

  let result: EndpointTypes[Routes.metadataNft]["response"] = { cip25: {} };
  const chunkedResult = await Promise.all(
    chunk(pairs, ASSET_LIMIT.REQUEST).map((chunk) =>
      query(urlBase, Routes.metadataNft, pairsToAssetMap(chunk))
    )
  );
  for (const chunk of chunkedResult) {
    result = merge(result, chunk);
  }

  return result;
}

export function nftCborToJson(
  request: EndpointTypes[Routes.metadataNft]["response"],
  cmlTransactioMetadatum: typeof cml.TransactionMetadatum,
  decode_metadatum_to_json_str: typeof cml.decode_metadatum_to_json_str,
  conversionType: cml.MetadataJsonSchema
): EndpointTypes[Routes.metadataNft]["response"] {
  const result: EndpointTypes[Routes.metadataNft]["response"]["cip25"] = {};
  for (const [policyId, assetNames] of Object.entries(request.cip25)) {
    const newAssetNameMap: Record<string, string> = {};
    for (const [assetName, cbor] of Object.entries(assetNames)) {
      const metadatum = cmlTransactioMetadatum.from_cbor_bytes(
        Buffer.from(cbor, "hex")
      );
      const json = decode_metadatum_to_json_str(metadatum, conversionType);
      newAssetNameMap[assetName] = json;
    }
    result[policyId] = newAssetNameMap;
  }

  return { cip25: result };
}
