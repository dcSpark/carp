/**
 * @pattern [0-9a-fA-F]{64}
 * @example "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a"
 */
export type CredentialHex = string;
/**
 * @example "stake1ux236z4g4r4pztn5v69txyj2yq6a3esq5x4p4stxydra7zsnv25ue"
 * @example "addr1q9ya8v4pe33nlkgftyd70nhhp407pvnjjcsddhf64sh9gegwtvyxm7r69gx9cwvtg82p87zpwmzj0kj7tjmyraze3pzqe6zxzv"
 */
export type Bech32FullAddress = string;
/**
 * @example "script1ffv7hkf75573h0mlsg3jc7cpyuq2pn6tk7xc08dtkx3q5ah7h47"
 */
export type Bech32Credential = string;
/**
 * @example "Ae2tdPwUPEZHu3NZa6kCwet2msq4xrBXKHBDvogFKwMsF18Jca8JHLRBas7"
 * @example "DdzFFzCqrht3UrnL3bCK5QMi9XtmkqGG3G2tmuY17tWyhq63S7EzMpJPogoPKx15drcnJkH4A7QdqYgs4h3XD1zXb3zkDyBuAZcaqYDS"
 */
export type Base58Address = string;

/**
 * Supported types:
 * - Credential hex (8200581c...) - note this is not a keyhash (it contains a credential type prefix)
 * - Bech32 full address (`addr` / `addr_test` / `stake` / `stake_test`)
 * - Bech32 credentials ( `addr_vkh`, `script`, etc.) - this is the recommended approach
 * - Legacy Byron format (Ae2, Dd, etc.)
 *
 * Note: we recommend avoiding to query base addresses history using bech32
 * As Cardano UTXO spendability depends only on the payment credential and not the full base address
 * The result will also miss transactions that are only related to the payment key of the address
 * ex: the payment key is used in a multisig
 *
 * Note: using two different address representations in the same query will hurt performance (ex: addr1 and addr_vkh1)
 * This because under-the-hood this will run multiple independent SQL queries for the different formats
 *
 * Warning: querying reward bech32 addresses is equivalent to querying the stake credential inside it
 * This may return more results than expected (ex: a multisig containing the staking key of the wallet)
 *
 * @example "addr1qxzksn47upfu4fwqfmxx29rn5znlkw3ag98ul8rgndwm79aaql88xw6ez84k2ln6lawnt79sdqh7qwq0wcs672auktmsawshfe"
 */
export type Address =
  | CredentialHex
  | Bech32FullAddress
  | Bech32Credential
  | Base58Address;

export type Credential = CredentialHex | Bech32Credential;
export type DisplayFormatAddress = Base58Address | Bech32FullAddress;
