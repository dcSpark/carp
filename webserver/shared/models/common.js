"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RelationFilterType = void 0;
// note: keep in sync with Rust type TxCredentialRelationValue
var RelationFilterType;
(function (RelationFilterType) {
    RelationFilterType[RelationFilterType["FILTER_ALL"] = 0] = "FILTER_ALL";
    RelationFilterType[RelationFilterType["Witness"] = 1] = "Witness";
    RelationFilterType[RelationFilterType["Input"] = 2] = "Input";
    RelationFilterType[RelationFilterType["Output"] = 4] = "Output";
    RelationFilterType[RelationFilterType["StakeDeregistration"] = 8] = "StakeDeregistration";
    RelationFilterType[RelationFilterType["StakeDelegation"] = 16] = "StakeDelegation";
    RelationFilterType[RelationFilterType["StakeRegistration"] = 32] = "StakeRegistration";
    RelationFilterType[RelationFilterType["DelegationTarget"] = 64] = "DelegationTarget";
    RelationFilterType[RelationFilterType["PoolOwner"] = 128] = "PoolOwner";
    RelationFilterType[RelationFilterType["PoolOperator"] = 256] = "PoolOperator";
    RelationFilterType[RelationFilterType["PoolReward"] = 512] = "PoolReward";
    RelationFilterType[RelationFilterType["MirRecipient"] = 1024] = "MirRecipient";
    RelationFilterType[RelationFilterType["Withdrawal"] = 2048] = "Withdrawal";
    RelationFilterType[RelationFilterType["RequiredSigner"] = 4096] = "RequiredSigner";
    RelationFilterType[RelationFilterType["InNativeScript"] = 8192] = "InNativeScript";
    RelationFilterType[RelationFilterType["UnusedInput"] = 16384] = "UnusedInput";
    RelationFilterType[RelationFilterType["UnusedInputStake"] = 32768] = "UnusedInputStake";
    RelationFilterType[RelationFilterType["InputStake"] = 65536] = "InputStake";
    RelationFilterType[RelationFilterType["OutputStake"] = 131072] = "OutputStake";
    RelationFilterType[RelationFilterType["NO_FILTER"] = 255] = "NO_FILTER";
})(RelationFilterType = exports.RelationFilterType || (exports.RelationFilterType = {}));
