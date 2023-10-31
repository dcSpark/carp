import {Amount, UtxoPointer} from "./common";

export type ProjectedNftRangeRequest = {
    range: { minSlot: number, maxSlot: number }
};

export type ProjectedNftRangeResponse = {
    txId: string | null,
    outputIndex: number,
    slot: number,
    asset: string,
    amount: string,
    status: string | null,
    plutusDatum: string | null,
}[];