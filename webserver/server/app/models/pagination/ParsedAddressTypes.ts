export type ParsedAddressTypes = {
  credentialHex: string[];
  exactAddress: string[];
  exactLegacyAddress: string[];
  invalid: string[];
  reverseMap: Map<string, Set<string>>;
};
