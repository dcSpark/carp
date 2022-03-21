import express from "express";
import pg from "pg";

const server = async () => {
  const db = new pg.Client(process.env.DATABASE_URL);
  await db.connect();

  const app = express();

  app.get("/transactions-history-for-addresses", async (req, res) => {
    if (!req.query.addresses) {
      res.status(400).json({ message: "addresses is required in query" });

      return;
    }

    const queryResult = await db.query(
      `
      with t as (
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
        WHERE "StakeCredential".credential = ANY ($1)
        ORDER BY "Block".height DESC,
          "Transaction".tx_index ASC
      )
      select json_agg(t)
      from t
    `,
      [req.query.addresses]
    );

    res.status(200).json({ data: queryResult.rows[0].json_agg || [] });
  });

  app.get("/check-addresses-in-use", (req, res) => {});

  app.get("/utxos-for-transactions", async (req, res) => {
    if (!req.query.transactions) {
      res.status(400).json({ message: "transactions is required in query" });

      return;
    }

    const queryResult = await db.query(
      `
      with t as (
        SELECT "TransactionOutput".id,
          "TransactionOutput".payload,
          "TransactionOutput".address_id,
          "TransactionOutput".tx_id,
          "TransactionOutput".output_index
        FROM "TransactionOutput"
        INNER JOIN "Transaction" ON "Transaction".id = "TransactionOutput".tx_id
        WHERE "Transaction".hash = ANY ($1)
      )
      select json_agg(t)
      from t
    `,
      [req.query.transactions]
    );

    res.status(200).json({ data: queryResult.rows[0].json_agg || [] });
  });

  app.get("/best-block", async (req, res) => {
    const queryResult = await db.query(
      'select id,height,hash from "Block" order by height desc LIMIT 1'
    );

    res.status(200).json({ data: queryResult.rows[0] });
  });

  app.listen(4000, () => {
    console.log("Server running on http://localhost:4000 🚀");
  });
};

server();
