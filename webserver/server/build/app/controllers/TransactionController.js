"use strict";
var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata = (this && this.__metadata) || function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
var __param = (this && this.__param) || function (paramIndex, decorator) {
    return function (target, key) { decorator(target, key, paramIndex); }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransactionController = void 0;
const tsoa_1 = require("tsoa");
const TransactionHistoryService_1 = require("../services/TransactionHistoryService");
const http_status_codes_1 = require("http-status-codes");
let TransactionController = class TransactionController extends tsoa_1.Controller {
    async txHistoryForAddresses(requestBody, errorResponse) {
        return await (0, TransactionHistoryService_1.countTxs)(
        // TODO: this is not what the real logic should be
        requestBody.addresses.map(addr => Buffer.from(addr, 'hex')));
    }
};
__decorate([
    (0, tsoa_1.SuccessResponse)(`${http_status_codes_1.StatusCodes.OK}`, 'Created'),
    (0, tsoa_1.Post)(),
    __param(0, (0, tsoa_1.Body)()),
    __param(1, (0, tsoa_1.Res)()),
    __metadata("design:type", Function),
    __metadata("design:paramtypes", [Object, Function]),
    __metadata("design:returntype", Promise)
], TransactionController.prototype, "txHistoryForAddresses", null);
TransactionController = __decorate([
    (0, tsoa_1.Route)('transactions')
], TransactionController);
exports.TransactionController = TransactionController;
