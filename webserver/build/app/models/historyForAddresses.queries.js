"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.historyForAddresses = void 0;
/** Types generated for queries found in "app/models/historyForAddresses.sql" */
const query_1 = require("@pgtyped/query");
const historyForAddressesIR = { "name": "HistoryForAddresses", "params": [], "usedParamSet": {}, "statement": { "body": "SELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".height\n      FROM \"StakeCredential\"\n      INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n      INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n      INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n      WHERE \"StakeCredential\".credential = ANY ($1)\n      ORDER BY \"Block\".height ASC,\n        \"Transaction\".tx_index ASC\n      LIMIT 100", "loc": { "a": 32, "b": 617, "line": 2, "col": 0 } } };
/**
 * Query generated from SQL:
 * ```
 * SELECT "Transaction".id,
 *         "Transaction".payload,
 *         "Transaction".hash,
 *         "Transaction".tx_index,
 *         "Transaction".is_valid,
 *         "Block".height
 *       FROM "StakeCredential"
 *       INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *       INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *       INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *       WHERE "StakeCredential".credential = ANY ($1)
 *       ORDER BY "Block".height ASC,
 *         "Transaction".tx_index ASC
 *       LIMIT 100
 * ```
 */
exports.historyForAddresses = new query_1.PreparedQuery(historyForAddressesIR);
