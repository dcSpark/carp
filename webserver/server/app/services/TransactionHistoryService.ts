import { historyForAddresses } from '../models/historyForAddresses.queries';
import type { TransactionHistoryResponse } from '../models/TransactionHistory';
import pool from './PgPoolSingleton';

export async function countTxs(stakeCredentials: Buffer[]): Promise<TransactionHistoryResponse> {
  const txs = await historyForAddresses.run(
    {
      // credentials: stakeCredentials.map(payload => `\\x${Buffer.from(payload).toString('hex')}`),
      credentials: stakeCredentials,
    },
    pool
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        num: entry.height,
        hash: entry.hash.toString('hex'),
        epoch: entry.epoch,
        slot: entry.slot,
        era: entry.era,
        tx_ordinal: entry.tx_index,
        is_valid: entry.is_valid,
      },
      transaction: entry.payload.toString('hex'),
    })),
  };
}
