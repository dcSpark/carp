import { historyForAddresses } from '../models/historyForAddresses.queries';
import type { TransactionHistoryResponse } from '../models/TransactionHistory';
import pool from './PgPoolSingleton';

// with t as (
//         SELECT "Transaction".id,
//           "Transaction".payload,
//           "Transaction".hash,
//           "Transaction".tx_index,
//           "Transaction".is_valid,
//           "Block".height
//         FROM "StakeCredential"
//         INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
//         INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
//         INNER JOIN "Block" ON "Transaction".block_id = "Block".id
//         WHERE "StakeCredential".credential = ANY ($1)
//         ORDER BY "Block".height ASC,
//           "Transaction".tx_index ASC
//         LIMIT 100
//       )
//       select json_agg(t)
//       from t
export async function countTxs(stakeCredentials: Buffer[]): Promise<TransactionHistoryResponse> {
  const txs = await historyForAddresses.run(undefined, pool);
  return {
    transactions: txs.map(
      entry =>
        ({
          block: {
            // num: entry.Block.height,
            // hash: entry.Block.hash.toString('hex'),
            // epoch: entry.Block.epoch,
            // slot: entry.Block.slot,
            // era: entry.Block.era,
            tx_ordinal: entry.tx_index,
            is_valid: entry.is_valid,
          },
          transaction: entry.payload.toString('hex'),
        } as any)
    ),
  };
}
