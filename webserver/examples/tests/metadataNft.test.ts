import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";

const urlBase = "http://localhost:3000";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.metadataNft}`, function () {
  this.timeout(10000);

  it("should find cip25 NFT metadata", async function () {
    const result = await query(urlBase, Routes.metadataNft, {
      assets: {
        // af2e27f580f7f08e93190a81f72462f153026d06450924726645891b: ["44524950"],
        b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f: [
          "42657272794e617679",
          "426572727954757271756f697365",
        ],
      },
    });

    const metadatum = cml.TransactionMetadatum.from_bytes(
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

    // TODO: tests for other tokens, test for missing tokens, tests for errors
    expect(JSON.parse(json)).to.be.eql({
      color: "#000080",
      image: "ipfs://ipfs/QmSKY1g1zSuPk56cXi2K8RNviaRkDHV3PZujttf7Ugk43y",
      name: "Berry Navy",
    });
  });
});
