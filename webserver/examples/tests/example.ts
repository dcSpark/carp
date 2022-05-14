import { Routes } from "@dcspark/carp-client/shared/routes";
import { paginatedTransactionHistory } from "@dcspark/carp-client/client/src/paginated";
import { query } from "@dcspark/carp-client/client/src/index";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import type { TransactionHistoryResponse } from "@dcspark/carp-client/shared/models/TransactionHistory";

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
  const paymentKey = wasmAddr.as_base()?.payment_cred();
  if (paymentKey == null) throw new Error();

  const stakingKey = wasmAddr.as_base()?.stake_cred();
  if (stakingKey == null) throw new Error();

  const result = await paginatedTransactionHistory(urlBase, {
    addresses: [
      Buffer.from(paymentKey.to_bytes()).toString("hex"),
      Buffer.from(stakingKey.to_bytes()).toString("hex"),
    ],
    untilBlock: bestBlock.block.hash,
  });
  return result;
}
