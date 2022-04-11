"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.countTxs = void 0;
const historyForAddresses_queries_1 = require("../models/historyForAddresses.queries");
const PgPoolSingleton_1 = __importDefault(require("./PgPoolSingleton"));
// with t as (
//         SELECT "Transaction".id,
//           "Transaction".payload,
//           "Transaction".hash,
//           "Transaction".tx_index,
//           "Transaction".is_valid,
//           "Block".height
//         FROM "StakeCredential"
//         INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
//         INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
//         INNER JOIN "Block" ON "Transaction".block_id = "Block".id
//         WHERE "StakeCredential".credential = ANY ($1)
//         ORDER BY "Block".height ASC,
//           "Transaction".tx_index ASC
//         LIMIT 100
//       )
//       select json_agg(t)
//       from t
async function countTxs(stakeCredentials) {
    const txs = await historyForAddresses_queries_1.historyForAddresses.run(undefined, PgPoolSingleton_1.default);
    return {
        transactions: txs.map(entry => ({
            block: {
                // num: entry.Block.height,
                // hash: entry.Block.hash.toString('hex'),
                // epoch: entry.Block.epoch,
                // slot: entry.Block.slot,
                // era: entry.Block.era,
                tx_ordinal: entry.tx_index,
                is_valid: entry.is_valid,
            },
            transaction: entry.payload.toString('hex'),
        })),
    };
}
exports.countTxs = countTxs;
