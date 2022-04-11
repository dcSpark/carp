"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.historyForAddresses = void 0;
/** Types generated for queries found in "app/models/historyForAddresses.sql" */
const query_1 = require("@pgtyped/query");
const historyForAddressesIR = { "name": "HistoryForAddresses", "params": [], "usedParamSet": {}, "statement": { "body": "SELECT * from \"Transaction\" LIMIT 100", "loc": { "a": 32, "b": 68, "line": 2, "col": 0 } } };
/**
 * Query generated from SQL:
 * ```
 * SELECT * from "Transaction" LIMIT 100
 * ```
 */
exports.historyForAddresses = new query_1.PreparedQuery(historyForAddressesIR);
