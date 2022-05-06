import type { UtxoPointer } from "./common";

export type TransactionOutputRequest = {
  utxoPointers: UtxoPointer[];
};

export type TransactionOutputResponse = {
  utxos: (UtxoPointer & { payload: string })[];
};
