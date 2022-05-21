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
export type Address = string;
