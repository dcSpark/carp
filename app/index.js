import express from "express";

const app = express();

app.get("/transactions-history-for-addresses", (req, res) => {});
app.get("/check-addresses-in-use", (req, res) => {});
app.get("/utxos-for-transactions", (req, res) => {});
app.get("/best-block", (req, res) => {});

app.listen(3000);
