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
    utxos: utxos.map((utxo, i) => ({
      txHash: request.utxoPointers[i].txHash,
      index: request.utxoPointers[i].index,
      payload: utxo.payload.toString('hex'),
    })),
  };
}
