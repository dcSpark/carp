import type { TransactionHistoryResponse } from '../models/TransactionHistory';
import * as db from 'zapatos/db';
import type * as s from 'zapatos/schema';
import type { Pool } from 'pg';

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
export async function countTxs(
  pool: Pool,
  stakeCredentials: Buffer[]
): Promise<TransactionHistoryResponse> {
  const tranasctions = await db.sql<
    s.Transaction.SQL,
    s.Transaction.Selectable[]
  >`SELECT ${'Transaction'}${'id'} FROM ${'Transaction'}`.run(pool);

  return {
    transactions: [
      {
        block: null,
        transaction: '',
      },
    ],
  };
}
