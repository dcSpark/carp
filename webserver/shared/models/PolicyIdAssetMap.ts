import type { UnboundHex } from "./common";

/**
 * @pattern [0-9a-fA-F]{56}
 * @example "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f"
 */
export type PolicyId = string;
/**
 * @pattern [0-9a-fA-F]{0,64}
 * @example "42657272794e617679"
 */
export type AssetName = string;
/**
 * @example "a365636f6c6f72672330303030383065696d616765783a697066733a2f2f697066732f516d534b593167317a5375506b3536635869324b38524e766961526b44485633505a756a7474663755676b343379646e616d656a4265727279204e617679"
 */
type Cip25Metadata = UnboundHex;

export type PolicyIdAssetMapType = {
  // https://github.com/lukeautry/tsoa/issues/1204#issuecomment-1133229741
  /**
   * @example { "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f": ["42657272794e617679"] }
   */
  assets: { [policyId: string]: AssetName[] };
};
export type Cip25Response = {
  // https://github.com/lukeautry/tsoa/issues/1204#issuecomment-1133229741
  /**
   * @example { "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f": { "42657272794e617679": "a365636f6c6f72672330303030383065696d616765783a697066733a2f2f697066732f516d534b593167317a5375506b3536635869324b38524e766961526b44485633505a756a7474663755676b343379646e616d656a4265727279204e617679" }}
   */
  cip25: { [policyId: string]: { [assetName: string]: Cip25Metadata } };
  // cip25: Record<PolicyId, Record<AssetName, Cip25Metadata>>;
};
export type NativeAsset = [policyId: PolicyId, assetName: AssetName];
