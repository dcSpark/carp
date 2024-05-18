import { expect } from "chai";
import { Errors } from "@dcspark/carp-client";
import { Routes } from "@dcspark/carp-client";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client";

const urlBase = "http://localhost:3000";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.blockLatest}`, function () {
  this.timeout(10000);

  it("should find the latest block possibly with an offset", async function () {
    const result = await query(urlBase, Routes.blockLatest, {
      offset: 0,
    });
    const resultOffByOne = await query(urlBase, Routes.blockLatest, {
      offset: 1,
    });
    expect(result.block.height).be.greaterThan(1);
    expect(result.block.height).be.greaterThan(resultOffByOne.block.height);
  });

  it("should error if the offset is too large", async function () {
    try {
      await query(urlBase, Routes.blockLatest, {
        offset: 1_000_000_000,
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.BAD_REQUEST);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.BlockOffsetLimit.prefix)
      );
    }
  });
});
