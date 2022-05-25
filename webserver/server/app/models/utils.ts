import { bech32 } from 'bech32';
import {
  Address,
  ByronAddress,
  Ed25519KeyHash,
  ScriptHash,
  StakeCredential,
} from '@dcspark/cardano-multiplatform-lib-nodejs';
import Cip5 from '@dcspark/cip5-js';
import type { ParsedAddressTypes } from './pagination/ParsedAddressTypes';

export const getAddressTypes = (addresses: string[]): ParsedAddressTypes => {
  const result: ParsedAddressTypes = {
    credentialHex: [],
    exactAddress: [],
    exactLegacyAddress: [],
    reverseMap: new Map(),
    invalid: [],
  };

  const updateSet = (key: string, value: string): void => {
    const set = result.reverseMap.get(key) ?? new Set();
    set.add(value);
    result.reverseMap.set(key, set);
  };

  for (const address of addresses) {
    const asCredHex = getAsCredentialHex(address);
    if (asCredHex != null) {
      result.credentialHex.push(asCredHex);
      updateSet(asCredHex, address);
      continue;
    }
    const asExactAddressHex = getAsCredentialHex(address);
    if (asExactAddressHex != null) {
      result.credentialHex.push(asExactAddressHex);
      updateSet(asExactAddressHex, address);
      continue;
    }

    if (ByronAddress.is_valid(address)) {
      const byronAddr = ByronAddress.from_base58(address);
      const asHex = Buffer.from(byronAddr.to_bytes()).toString('hex');

      updateSet(asHex, address);
      result.exactLegacyAddress.push(asHex);

      byronAddr.free();
      continue;
    }
    result.invalid.push(address);
  }

  return result;
};

const credentialLength = 32 * 2; // 32 bytes = 64 hex letters
const credentialRegex = new RegExp(`^[0-9a-fA-F]{${credentialLength}}$`);
export function isCredentialHex(maybeCredentialHex: string): boolean {
  return credentialRegex.test(maybeCredentialHex);
}

export function getAsCredentialHex(address: string): undefined | string {
  if (isCredentialHex(address)) {
    return address;
  }
  try {
    const bech32Info = bech32.decode(address, 1000);
    switch (bech32Info.prefix) {
      case Cip5.miscellaneous.stake:
      case Cip5.miscellaneous.stake_test: {
        const addr = Address.from_bech32(address);
        const rewardAddr = addr.as_reward();
        if (rewardAddr == null) {
          addr.free();
        } else {
          const cred = rewardAddr.payment_cred();
          const asHex = Buffer.from(cred.to_bytes()).toString('hex');

          addr.free();
          cred.free();

          return asHex;
        }
      }
      case Cip5.hashes.addr_vkh:
      case Cip5.hashes.policy_vkh:
      case Cip5.hashes.stake_vkh:
      case Cip5.hashes.stake_shared_vkh:
      case Cip5.hashes.addr_shared_vkh: {
        const payload = bech32.fromWords(bech32Info.words);
        const keyHash = Ed25519KeyHash.from_bytes(Buffer.from(payload));
        const stakeCred = StakeCredential.from_keyhash(keyHash);
        const asHex = Buffer.from(stakeCred.to_bytes()).toString('hex');

        keyHash.free();
        stakeCred.free();

        return asHex;
      }
      case Cip5.hashes.script: {
        const payload = bech32.fromWords(bech32Info.words);
        const keyHash = ScriptHash.from_bytes(Buffer.from(payload));
        const stakeCred = StakeCredential.from_scripthash(keyHash);
        const asHex = Buffer.from(stakeCred.to_bytes()).toString('hex');

        keyHash.free();
        stakeCred.free();

        return asHex;
      }
    }
  } catch (_e) {}
  return undefined;
}

export function getAsExactAddressHex(address: string): undefined | string {
  try {
    const bech32Info = bech32.decode(address, 1000);
    switch (bech32Info.prefix) {
      case Cip5.miscellaneous.addr:
      case Cip5.miscellaneous.addr_test:
        const payload = bech32.fromWords(bech32Info.words);
        const asHex = Buffer.from(payload).toString('hex');

        return asHex;
    }
  } catch (_e) {}
  return undefined;
}
