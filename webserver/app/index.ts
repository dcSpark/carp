import express from 'express';
import type { Response as ExResponse, Request as ExRequest } from 'express';
import swaggerUi from 'swagger-ui-express';
import bodyParser from 'body-parser';
import { RegisterRoutes } from '../build/routes';
import SwaggerSingleton from './models/SwaggerSingleton';

export const app = express();

app.use(
  bodyParser.urlencoded({
    extended: true,
  })
);
app.use(bodyParser.json());

app.use('/docs', swaggerUi.serve, async (_req: ExRequest, res: ExResponse) => {
  return res.send(swaggerUi.generateHTML(await SwaggerSingleton()));
});

// app.use(function notFoundHandler(_req, res: ExResponse) {
//   res.status(404).send({
//     message: 'Not Found',
//   });
// });

RegisterRoutes(app);

// main()
//   .catch(e => {
//     throw e;
//   })

//   .finally(async () => {
//     await prisma.$disconnect();
//   });

// const server = async () => {
//   const db = new pg.Client(process.env.DATABASE_URL);
//   await db.connect();

//   app.get('/transactions-history-for-addresses', async (req, res) => {
//     if (!req.query.addresses) {
//       res.status(400).json({ message: 'addresses is required in query' });

//       return;
//     }

//     const queryResult = await db.query(
//       `
//       with t as (
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
//     `,
//       [req.query.addresses]
//     );

//     res.status(200).json({ data: queryResult.rows[0].json_agg || [] });
//   });

//   app.get('/check-addresses-in-use', async (req, res) => {
//     const queryResult = await db.query(
//       `
//       SELECT "StakeCredential".credential
//       FROM "StakeCredential"
//       INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
//       INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
//       WHERE "StakeCredential".credential = ANY ($1)
//       `,
//       [req.query.addresses]
//     );

//     res.status(200).json({ data: queryResult.rows || [] });
//   });

//   app.get('/utxos-for-transactions', async (req, res) => {
//     if (!req.query.transactions) {
//       res.status(400).json({ message: 'transactions is required in query' });

//       return;
//     }

//     const queryResult = await db.query(
//       `
//       with t as (
//         SELECT "TransactionOutput".id,
//           "TransactionOutput".payload,
//           "TransactionOutput".address_id,
//           "TransactionOutput".tx_id,
//           "TransactionOutput".output_index
//         FROM "TransactionOutput"
//         INNER JOIN "Transaction" ON "Transaction".id = "TransactionOutput".tx_id
//         WHERE "Transaction".hash = ANY ($1)
//       )
//       select json_agg(t)
//       from t
//     `,
//       [req.query.transactions]
//     );

//     res.status(200).json({ data: queryResult.rows[0].json_agg || [] });
//   });

//   app.get('/best-block', async (req, res) => {
//     const queryResult = await db.query(
//       'select id,height,hash from "Block" order by height desc LIMIT 1'
//     );

//     res.status(200).json({ data: queryResult.rows[0] });
//   });

//   app.listen(4000, () => {
//     console.log('Server running on http://localhost:4000 🚀');
//   });
// };
