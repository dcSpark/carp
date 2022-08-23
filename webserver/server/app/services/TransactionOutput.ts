import type { Pool } from 'pg';
import { sqlTransactionOutput } from '../models/transaction/sqlTransactionOutput.queries';
import type {
  TransactionOutputRequest,
  TransactionOutputResponse,
} from '../../../shared/models/TransactionOutput';

export async function outputsForTransaction(
  request: TransactionOutputRequest & {
    dbTx: Pool;
  }
): Promise<TransactionOutputResponse> {
  if (request.utxoPointers.length === 0) return { utxos: [] };
  const utxos = await sqlTransactionOutput.run(
    {
      tx_hash: request.utxoPointers.map(pointer => Buffer.from(pointer.txHash, 'hex')),
      output_index: request.utxoPointers.map(pointer => pointer.index),
    },
    request.dbTx
  );
  return {
    utxos: utxos.map(db_utxo => ({
      utxo: {
        txHash: db_utxo.hash.toString('hex'),
        index: db_utxo.output_index,
        payload: db_utxo.utxo_payload.toString('hex'),
      },
      block: {
        height: db_utxo.height,
        hash: db_utxo.block_hash.toString('hex'),
        epoch: db_utxo.epoch,
        slot: db_utxo.slot,
        era: db_utxo.era,
        indexInBlock: db_utxo.tx_index,
        isValid: db_utxo.is_valid,
      },
    })),
  };
}
