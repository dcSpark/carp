import type { TransactionHistoryResponse } from '../../../shared/models/TransactionHistory';
import { historyForAddresses } from '../models/historyForAddresses.queries';
import pool from './PgPoolSingleton';

export async function historyForAddress(
  stakeCredentials: Buffer[]
): Promise<TransactionHistoryResponse> {
  const txs = await historyForAddresses.run(
    {
      credentials: stakeCredentials,
    },
    pool
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        num: entry.height,
        hash: entry.block_hash.toString('hex'),
        epoch: entry.epoch,
        slot: entry.slot,
        era: entry.era,
        tx_ordinal: entry.tx_index,
        is_valid: entry.is_valid,
      },

      transaction: {
        hash: entry.hash.toString('hex'),
        payload: entry.payload.toString('hex'),
      },
    })),
  };
}
