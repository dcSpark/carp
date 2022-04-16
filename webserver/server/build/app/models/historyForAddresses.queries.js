"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.historyForAddresses = void 0;
/** Types generated for queries found in "app/models/historyForAddresses.sql" */
const query_1 = require("@pgtyped/query");
const historyForAddressesIR = { "name": "HistoryForAddresses", "params": [{ "name": "credentials", "required": false, "transform": { "type": "scalar" }, "codeRefs": { "used": [{ "a": 596, "b": 606, "line": 15, "col": 49 }] } }], "usedParamSet": { "credentials": true }, "statement": { "body": "SELECT \"Transaction\".id,\n        \"Transaction\".payload,\n        \"Transaction\".hash,\n        \"Transaction\".tx_index,\n        \"Transaction\".is_valid,\n        \"Block\".epoch,\n        \"Block\".slot,\n        \"Block\".era,\n        \"Block\".height\n      FROM \"StakeCredential\"\n      INNER JOIN \"TxCredentialRelation\" ON \"TxCredentialRelation\".credential_id = \"StakeCredential\".id\n      INNER JOIN \"Transaction\" ON \"TxCredentialRelation\".tx_id = \"Transaction\".id\n      INNER JOIN \"Block\" ON \"Transaction\".block_id = \"Block\".id\n      WHERE \"StakeCredential\".credential = ANY (:credentials)\n      ORDER BY \"Block\".height ASC,\n        \"Transaction\".tx_index ASC\n      LIMIT 100", "loc": { "a": 32, "b": 693, "line": 2, "col": 0 } } };
/**
 * Query generated from SQL:
 * ```
 * SELECT "Transaction".id,
 *         "Transaction".payload,
 *         "Transaction".hash,
 *         "Transaction".tx_index,
 *         "Transaction".is_valid,
 *         "Block".epoch,
 *         "Block".slot,
 *         "Block".era,
 *         "Block".height
 *       FROM "StakeCredential"
 *       INNER JOIN "TxCredentialRelation" ON "TxCredentialRelation".credential_id = "StakeCredential".id
 *       INNER JOIN "Transaction" ON "TxCredentialRelation".tx_id = "Transaction".id
 *       INNER JOIN "Block" ON "Transaction".block_id = "Block".id
 *       WHERE "StakeCredential".credential = ANY (:credentials)
 *       ORDER BY "Block".height ASC,
 *         "Transaction".tx_index ASC
 *       LIMIT 100
 * ```
 */
exports.historyForAddresses = new query_1.PreparedQuery(historyForAddressesIR);
