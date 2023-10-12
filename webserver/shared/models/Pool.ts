/**
 * @pattern [0-9a-fA-F]{56}
 * @example "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a"
 */
export type PoolHex = string;

export type Pool =
  | PoolHex