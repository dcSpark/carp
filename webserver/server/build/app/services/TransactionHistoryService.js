"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.countTxs = void 0;
const historyForAddresses_queries_1 = require("../models/historyForAddresses.queries");
const PgPoolSingleton_1 = __importDefault(require("./PgPoolSingleton"));
async function countTxs(stakeCredentials) {
    const txs = await historyForAddresses_queries_1.historyForAddresses.run({
        // credentials: stakeCredentials.map(payload => `\\x${Buffer.from(payload).toString('hex')}`),
        credentials: stakeCredentials,
    }, PgPoolSingleton_1.default);
    return {
        transactions: txs.map(entry => ({
            block: {
                num: entry.height,
                hash: entry.hash.toString('hex'),
                epoch: entry.epoch,
                slot: entry.slot,
                era: entry.era,
                tx_ordinal: entry.tx_index,
                is_valid: entry.is_valid,
            },
            transaction: entry.payload.toString('hex'),
        })),
    };
}
exports.countTxs = countTxs;
