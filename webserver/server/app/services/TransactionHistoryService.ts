import type { TransactionHistoryResponse } from '../../../shared/models/TransactionHistory';
import { sqlHistoryForCredentials } from '../models/transaction/sqlHistoryForCredentials.queries';
import { sqlHistoryForAddresses } from '../models/transaction/sqlHistoryForAddresses.queries';
import type { PoolClient } from 'pg';
import type { TransactionPaginationType } from './PaginationService';
import type { RelationFilter } from '../../../shared/models/common';
import { Address, Transaction } from '@dcspark/cardano-multiplatform-lib-nodejs';

export async function historyForCredentials(
  request: TransactionPaginationType & {
    dbTx: PoolClient;
    stakeCredentials: Buffer[];
    relationFilter: RelationFilter;
    limit: number;
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
        hash: entry.hash.toString('hex'),
        payload: entry.payload.toString('hex'),
        outputs: computeOutputs(entry.payload),
        metadata: entry.metadata && entry.metadata.toString('hex'),
        inputCredentials: entry.input_addresses
          ? (entry.input_addresses as string[]).map(getPaymentCred)
          : [],
      },
    })),
  };
}

export async function historyForAddresses(
  request: TransactionPaginationType & {
    addresses: Buffer[];
    dbTx: PoolClient;
    limit: number;
  }
): Promise<TransactionHistoryResponse> {
  if (request.addresses?.length === 0) return { transactions: [] };
  const txs = await sqlHistoryForAddresses.run(
    {
      addresses: request.addresses,
      after_tx_id: (request.after?.tx_id ?? -1)?.toString(),
      limit: request.limit.toString(),
      until_tx_id: request.until.tx_id.toString(),
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
        hash: entry.hash.toString('hex'),
        payload: entry.payload.toString('hex'),
        outputs: computeOutputs(entry.payload),
        metadata: entry.metadata && entry.metadata.toString('hex'),
        inputCredentials: entry.input_addresses
          ? (entry.input_addresses as string[]).map(getPaymentCred)
          : [],
      },
    })),
  };
}

function computeOutputs(
  tx: Buffer
): { asset: { policyId: string; assetName: string } | null; amount: string; address: string }[] {
  const transaction = Transaction.from_bytes(tx);

  const rawOutputs = transaction.body().outputs();

  const outputs = [];

  for (let i = 0; i < rawOutputs.len(); i++) {
    const output = rawOutputs.get(i);

    const rawAddress = output.address();
    const address = rawAddress.to_bech32();
    rawAddress.free();

    const amount = output.amount();
    const ma = amount.multiasset();

    if (ma) {
      const policyIds = ma.keys();

      for (let j = 0; j < policyIds.len(); j++) {
        const policyId = policyIds.get(j);

        const assets = ma.get(policyId);

        if (!assets) {
          continue;
        }

        const assetNames = assets.keys();

        for (let k = 0; k < assetNames.len(); k++) {
          const assetName = assetNames.get(k);

          const amount = assets.get(assetName);

          if (amount === undefined) {
            continue;
          }

          outputs.push({
            amount: amount.to_str(),
            asset: {
              policyId: policyId.to_hex(),
              assetName: Buffer.from(assetName.to_bytes()).toString('hex'),
            },
            address
          });

          assetName.free();
        }

        assetNames.free();
        assets.free();
        policyId.free();
      }

      policyIds.free();
      ma.free();
    }

    outputs.push({ amount: amount.coin().to_str(), asset: null, address });

    amount.free();
    output.free();
  }

  rawOutputs.free();
  transaction.free();

  return outputs;
}

function getPaymentCred(addressRaw: string): string {
  const address = Address.from_bytes(Buffer.from(addressRaw.slice(2), 'hex'));

  const paymentCred = address.payment_cred();
  const addressBytes = paymentCred?.to_bytes();

  address.free();
  paymentCred?.free();

  return Buffer.from(addressBytes as Uint8Array).toString('hex');
}
