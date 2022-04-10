"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.countTxs = void 0;
const PrismaSingleton_1 = __importDefault(require("./PrismaSingleton"));
async function countTxs() {
    const numTxs = await PrismaSingleton_1.default.transaction.count();
    return numTxs;
}
exports.countTxs = countTxs;
