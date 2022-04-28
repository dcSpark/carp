import { ADDRESS_RESPONSE_LIMIT } from '../../../shared/constants';
import type { TransactionHistoryResponse } from '../../../shared/models/TransactionHistory';
import { sqlHistoryForCredentials } from '../models/sqlHistoryForCredentials.queries';
import { sqlHistoryForAddresses } from '../models/sqlHistoryForAddresses.queries';
import pool from './PgPoolSingleton';

export async function historyForCredentials(
  stakeCredentials: Buffer[]
): Promise<TransactionHistoryResponse> {
  if (stakeCredentials.length === 0) return { transactions: [] };
  const txs = await sqlHistoryForCredentials.run(
    {
      credentials: stakeCredentials,
      limit: ADDRESS_RESPONSE_LIMIT.toString(),
    },
    pool
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        height: entry.height,
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

export async function historyForAddresses(
  addresses: Buffer[]
): Promise<TransactionHistoryResponse> {
  if (addresses.length === 0) return { transactions: [] };
  const txs = await sqlHistoryForAddresses.run(
    {
      addresses: addresses,
      limit: ADDRESS_RESPONSE_LIMIT.toString(),
    },
    pool
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        height: entry.height,
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
