import type { UnboundHex, UtxoPointer } from "./common";
import { BlockInfo } from "./TransactionHistory";

export type TransactionOutputRequest = {
  utxoPointers: UtxoPointer[];
};

export type UtxoAndBlockInfo = {
  block: BlockInfo;
  utxo: UtxoPointer & {
    /**
     * @example "825839019cb581f4337a6142e477af6e00fe41b1fc4a5944a575681b8499a3c0bd07ce733b5911eb657e7aff5d35f8b0682fe0380f7621af2bbcb2f71b0000000586321393"
     */
    payload: UnboundHex;
  };
};
export type TransactionOutputResponse = {
  utxos: UtxoAndBlockInfo[];
};
