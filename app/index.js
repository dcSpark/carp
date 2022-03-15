import express from "express";
import pg from "pg";

const server = async () => {
  const db = new pg.Client(process.env.DATABASE_URL);
  await db.connect();

  const app = express();

  app.get("/transactions-history-for-addresses", (req, res) => {
    await db.query(
      `
      SELECT "Transaction".id,
        "Transaction".payload,
        "Transaction".hash,
        "Transaction".tx_index,
        "Transaction".is_valid,
        "Block".height
      FROM "StakeCredential"
      INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
      INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
      INNER JOIN "Block" ON "Transaction".block_id = "Block".id
      WHERE "StakeCredential".credential IN $1
      ORDER BY "Block".height DESC,
        "Transaction".tx_index ASC;
    `,
      [req.body.addresses]
    );
  });

  app.get("/check-addresses-in-use", (req, res) => {});

  app.get("/utxos-for-transactions", (req, res) => {});

  app.get("/best-block", (req, res) => {});

  app.listen(3000);
};

server();
