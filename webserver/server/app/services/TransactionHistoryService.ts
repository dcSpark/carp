import type { TransactionHistoryResponse } from '../../../shared/models/TransactionHistory';
import { sqlHistoryForCredentials } from '../models/transaction/sqlHistoryForCredentials.queries';
import { sqlHistoryForAddresses } from '../models/transaction/sqlHistoryForAddresses.queries';
import type { PoolClient } from 'pg';
import type { TransactionPaginationType } from './PaginationService';
import type { RelationFilter } from '../../../shared/models/common';
import { Address } from '@dcspark/cardano-multiplatform-lib-nodejs';

export async function historyForCredentials(
  request: TransactionPaginationType & {
    dbTx: PoolClient;
    stakeCredentials: Buffer[];
    relationFilter: RelationFilter;
    limit: number;
    withInputContext?: boolean;
  }
): Promise<TransactionHistoryResponse> {
  if (request.stakeCredentials.length === 0) return { transactions: [] };
  const txs = await sqlHistoryForCredentials.run(
    {
      credentials: request.stakeCredentials,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      limit: request.limit.toString(),
      until_tx_id: request.until.tx_id.toString(),
      relation: request.relationFilter,
      with_input_context: !!request.withInputContext,
    },
    request.dbTx
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        height: entry.height,
        hash: entry.block_hash.toString('hex'),
        epoch: entry.epoch,
        slot: entry.slot,
        era: entry.era,
        indexInBlock: entry.tx_index,
        isValid: entry.is_valid,
      },

      transaction: {
        ...{
          hash: entry.hash.toString('hex'),
          payload: entry.payload.toString('hex'),
        },
        ...(request.withInputContext && {
          metadata: entry.metadata && entry.metadata.toString('hex'),
          inputCredentials: entry.input_addresses
            ? (entry.input_addresses as string[]).map(getPaymentCred)
            : [],
        }),
      },
    })),
  };
}

export async function historyForAddresses(
  request: TransactionPaginationType & {
    addresses: Buffer[];
    dbTx: PoolClient;
    limit: number;
    withInputContext?: boolean;
  }
): Promise<TransactionHistoryResponse> {
  if (request.addresses?.length === 0) return { transactions: [] };
  const txs = await sqlHistoryForAddresses.run(
    {
      addresses: request.addresses,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      limit: request.limit.toString(),
      until_tx_id: request.until.tx_id.toString(),
      with_input_context: !!request.withInputContext,
    },
    request.dbTx
  );
  return {
    transactions: txs.map(entry => ({
      block: {
        height: entry.height,
        hash: entry.block_hash.toString('hex'),
        epoch: entry.epoch,
        slot: entry.slot,
        era: entry.era,
        indexInBlock: entry.tx_index,
        isValid: entry.is_valid,
      },

      transaction: {
        ...{
          hash: entry.hash.toString('hex'),
          payload: entry.payload.toString('hex'),
        },
        ...(request.withInputContext && {
          metadata: entry.metadata && entry.metadata.toString('hex'),
          inputCredentials: entry.input_addresses
            ? (entry.input_addresses as string[]).map(getPaymentCred)
            : [],
        }),
      },
    })),
  };
}

function getPaymentCred(addressRaw: string): string {
  const address = Address.from_raw_bytes(Buffer.from(addressRaw.slice(2), 'hex'));

  const paymentCred = address.payment_cred();
  const addressBytes = paymentCred?.to_cbor_bytes();

  return Buffer.from(addressBytes as Uint8Array).toString('hex');
}
