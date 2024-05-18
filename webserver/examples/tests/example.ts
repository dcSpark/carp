import { Routes } from "@dcspark/carp-client";
import {
  nftCborToJson,
  paginatedMetadataNft,
  paginatedTransactionHistory,
} from "@dcspark/carp-client";
import { query } from "@dcspark/carp-client";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import type { TransactionHistoryResponse } from "@dcspark/carp-client";

const urlBase = "http://localhost:3000";

async function getHistoryForAddress(
  bech32Address: string
): Promise<TransactionHistoryResponse> {
  const bestBlock = await query(urlBase, Routes.blockLatest, {
    // the higher you make this, the less you have to worry about rollbacks
    // but also the slower your app will react to new transactions by the user
    // you can look into projects like Cardano multiverse-rs to optimize this number
    offset: 3,
  });

  const wasmAddr = cml.Address.from_bech32(bech32Address);
  const paymentKey = cml.BaseAddress.from_address(wasmAddr)?.payment();
  if (paymentKey == null) throw new Error();

  const stakingKey = cml.BaseAddress.from_address(wasmAddr)?.stake();
  if (stakingKey == null) throw new Error();

  const result = await paginatedTransactionHistory(urlBase, {
    addresses: [
      // Note: querying both the payment key & staking key here is different from querying the base32 address directly
      // Both methods are supported by Carp (and multiple different kinds of inputs too)
      // Be sure to pick the method that best works for you as they will give different results
      Buffer.from(paymentKey.to_cbor_bytes()).toString("hex"),
      Buffer.from(stakingKey.to_cbor_bytes()).toString("hex"),
    ],
    untilBlock: bestBlock.block.hash,
  });
  return result;
}

async function getNftInfo(
  policyId: string,
  assetName: string
): Promise<string> {
  const nftInfo = await paginatedMetadataNft(urlBase, {
    assets: { [policyId]: [assetName] },
  });

  const jsonInfo = nftCborToJson(
    nftInfo,
    cml.TransactionMetadatum,
    cml.decode_metadatum_to_json_str,
    cml.MetadataJsonSchema.BasicConversions
  );

  return jsonInfo.cip25[policyId][assetName];
}
