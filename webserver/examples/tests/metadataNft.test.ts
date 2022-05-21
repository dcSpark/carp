import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";

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

    expect(result.cip25).to.be.eql({});
  });
});
