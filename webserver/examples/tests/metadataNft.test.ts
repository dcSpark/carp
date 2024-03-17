import { expect } from "chai";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { query } from "@dcspark/carp-client/client/src/index";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import merge from "lodash/merge";

const urlBase = "http://localhost:3000";

const Policies = {
  // token pretending to be an NFT
  DripDropz: "af2e27f580f7f08e93190a81f72462f153026d06450924726645891b",
  Berry: "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f",
  ClayNation: "40fa2aa67258b4ce7b5782f74831d46a84c59a0ff0c28262fab21728",
  CardanoBits: "1131301ad4b3cb7deaddbc8f03f77189082a5738c0167e1772233097",
};

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.metadataNft}`, function () {
  this.timeout(10000);

  it("should find cip25 NFT metadata", async function () {
    const existingNfts = {
      assets: {
        // af2e27f580f7f08e93190a81f72462f153026d06450924726645891b: ["44524950"],
        [Policies.Berry]: [
          "42657272794e617679",
          "426572727954757271756f697365",
        ],
      },
    };
    const missingNfts = {
      assets: {
        // policy doesn't exist (invalid length)
        b863bc7369f468adb2fa7dae3af944c3bbb2be2f216a8d4f: [
          "42657272794e617679",
        ],
        // policy doesn't exist (real length)
        "0000301ad4b3cb7deaddbc8f03f77189082a5738c0167e1772233097": ["0000"],
        // policy exists, asset doesn't
        [Policies.CardanoBits]: ["0000"],

        // policy exists, no asset specified
        [Policies.ClayNation]: [],
      },
    };

    const result = await query(
      urlBase,
      Routes.metadataNft,
      merge(existingNfts, missingNfts)
    );

    expect(result).to.be.eql({
      cip25: {
        [Policies.Berry]: {
          "42657272794e617679":
            "a365636f6c6f72672330303030383065696d616765783a697066733a2f2f697066732f516d534b593167317a5375506b3536635869324b38524e766961526b44485633505a756a7474663755676b343379646e616d656a4265727279204e617679",
          "426572727954757271756f697365":
            "a365636f6c6f72672333304435433865696d616765783a697066733a2f2f697066732f516d6168565446376e627254673264784a7973646e6e57784c7731675168674a53794e5159436d5158336d536567646e616d656f42657272792054757271756f697365",
        },
      },
    });

    const metadatum = cml.TransactionMetadatum.from_cbor_bytes(
      Buffer.from(
        result.cip25[
          "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f"
        ]["42657272794e617679"],
        "hex"
      )
    );
    const json = cml.decode_metadatum_to_json_str(
      metadatum,
      cml.MetadataJsonSchema.BasicConversions
    );

    expect(JSON.parse(json)).to.be.eql({
      color: "#000080",
      image: "ipfs://ipfs/QmSKY1g1zSuPk56cXi2K8RNviaRkDHV3PZujttf7Ugk43y",
      name: "Berry Navy",
    });
  });
});
