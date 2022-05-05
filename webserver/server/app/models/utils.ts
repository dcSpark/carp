import { bech32 } from 'bech32';
import type { ParsedAddressTypes } from './ParsedAddressTypes';
import {
  Address,
  ByronAddress,
  Ed25519KeyHash,
  ScriptHash,
  StakeCredential,
} from '@dcspark/cardano-multiplatform-lib-nodejs';
import Cip5 from '@dcspark/cip5-js';

const credentialLength = 32 * 2; // 32 bytes = 64 hex letters

export const getAddressTypes = (addresses: string[]): ParsedAddressTypes => {
  const result: ParsedAddressTypes = {
    credentialHex: [],
    exactAddress: [],
    exactLegacyAddress: [],
    invalid: [],
  };
  const isCredentialHex = (address: string) =>
    new RegExp(`^[0-9a-fA-F]{${credentialLength}}$`).test(address);
  for (const address of addresses) {
    if (isCredentialHex(address)) {
      result.credentialHex.push(address);
      continue;
    }
    try {
      const bech32Info = bech32.decode(address, 1000);
      switch (bech32Info.prefix) {
        case Cip5.miscellaneous.addr:
        case Cip5.miscellaneous.addr_test:
          const payload = bech32.fromWords(bech32Info.words);
          result.exactAddress.push(Buffer.from(payload).toString('hex'));
          continue;
        case Cip5.miscellaneous.stake:
        case Cip5.miscellaneous.stake_test: {
          const addr = Address.from_bech32(address);
          const rewardAddr = addr.as_reward();
          if (rewardAddr == null) {
            result.invalid.push(address);
            addr.free();
          } else {
            const cred = rewardAddr.payment_cred();
            result.credentialHex.push(Buffer.from(cred.to_bytes()).toString('hex'));
            addr.free();
            cred.free();
          }
          continue;
        }
        case Cip5.hashes.addr_vkh:
        case Cip5.hashes.policy_vkh:
        case Cip5.hashes.stake_vkh:
        case Cip5.hashes.stake_shared_vkh:
        case Cip5.hashes.addr_shared_vkh: {
          const payload = bech32.fromWords(bech32Info.words);
          const keyHash = Ed25519KeyHash.from_bytes(Buffer.from(payload));
          const stakeCred = StakeCredential.from_keyhash(keyHash);
          result.credentialHex.push(Buffer.from(stakeCred.to_bytes()).toString('hex'));
          keyHash.free();
          stakeCred.free();
          continue;
        }
        case Cip5.hashes.script: {
          const payload = bech32.fromWords(bech32Info.words);
          const keyHash = ScriptHash.from_bytes(Buffer.from(payload));
          const stakeCred = StakeCredential.from_scripthash(keyHash);
          result.credentialHex.push(Buffer.from(stakeCred.to_bytes()).toString('hex'));
          keyHash.free();
          stakeCred.free();
          continue;
        }
      }
    } catch (_e) {}
    if (ByronAddress.is_valid(address)) {
      const byronAddr = ByronAddress.from_base58(address);
      result.exactLegacyAddress.push(Buffer.from(byronAddr.to_bytes()).toString('hex'));
      byronAddr.free();
      continue;
    }
    result.invalid.push(address);
  }

  return result;
};
