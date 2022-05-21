type PolicyId = string;
type AssetName = string;
type Cip25Metadata = string;

export type PolicyIdAssetMapType = {
  // https://github.com/lukeautry/tsoa/issues/1204#issuecomment-1133229741
  assets: { [policyId: string]: AssetName[] };
};
export type Cip25Response = {
  cip25: Record<PolicyId, Record<AssetName, Cip25Metadata>>;
};
export type NativeAsset = [policyId: PolicyId, assetName: AssetName];
