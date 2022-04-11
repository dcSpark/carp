"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.countTxs = void 0;
const PrismaSingleton_1 = __importDefault(require("./PrismaSingleton"));
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
    // await prisma.$queryRaw`SELECT * FROM User`;
    // const foo = await prisma.transaction.findMany({
    //   select: {
    //     id: true,
    //     payload: true,
    //     hash: true,
    //     tx_index: true,
    //     is_valid: true,
    //     Block: {
    //       select: {
    //         height: true,
    //       },
    //     },
    //   },
    //   orderBy: [
    //     {
    //       Block: {
    //         height: 'asc',
    //       },
    //     },
    //     {
    //       tx_index: 'asc',
    //     },
    //   ],
    // });
    // const foo2 = await prisma.block.findMany({
    //   select: {
    //     height: true,
    //     Transaction: {
    //       select: {
    //         id: true,
    //         payload: true,
    //         hash: true,
    //         tx_index: true,
    //         is_valid: true,
    //       },
    //     },
    //   },
    //   orderBy: [
    //     {
    //       height: 'asc',
    //     },
    //     {
    //       Transaction: {
    //         tx_index: 'asc',
    //       },
    //     },
    //   ],
    // });
    // const foo = await prisma.transaction.findMany({
    //   include: {
    //     Block: true,
    //   },
    //   orderBy: [
    //     {
    //       Block: {
    //         height: 'asc',
    //       },
    //     },
    //     {
    //       tx_index: 'asc',
    //     },
    //   ],
    // });
    const txInfo = await PrismaSingleton_1.default.transaction.findMany({
        select: {
            id: true,
            payload: true,
            hash: true,
            tx_index: true,
            is_valid: true,
            Block: {
                select: {
                    height: true,
                    hash: true,
                    epoch: true,
                    slot: true,
                    era: true,
                },
            },
        },
        where: {
            TxCredentialRelation: {
                every: {
                    StakeCredential: {
                        is: {
                            credential: {
                                in: stakeCredentials,
                            },
                        },
                    },
                },
            },
        },
        orderBy: [
            {
                Block: {
                    height: 'asc',
                },
            },
            {
                tx_index: 'asc',
            },
        ],
        take: 100,
    });
    return {
        transactions: txInfo.map(entry => ({
            block: {
                num: entry.Block.height,
                hash: entry.Block.hash.toString('hex'),
                epoch: entry.Block.epoch,
                slot: entry.Block.slot,
                era: entry.Block.era,
                tx_ordinal: entry.tx_index,
                is_valid: entry.is_valid,
            },
            transaction: entry.payload.toString('hex'),
        })),
    };
}
exports.countTxs = countTxs;
