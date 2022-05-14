import type { UtxoPointer } from "./common";
import { BlockInfo } from "./TransactionHistory";

export type TransactionOutputRequest = {
  utxoPointers: UtxoPointer[];
};

export type UtxoAndBlockInfo = {
  block: BlockInfo;
  utxo: UtxoPointer & { payload: string };
};
export type TransactionOutputResponse = {
  utxos: UtxoAndBlockInfo[];
};
