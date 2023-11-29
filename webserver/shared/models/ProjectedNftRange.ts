export type ProjectedNftRangeRequest = {
    range: { minSlot: number, maxSlot: number }
};

export type ProjectedNftRangeResponse = {
    actionSlot: number,

    ownerAddress: string | null,

    actionTxId: string | null,

    previousTxHash: string | null,
    previousTxOutputIndex: number | null,

    asset: string,
    amount: number,
    status: string | null,
    plutusDatum: string | null,
}[];