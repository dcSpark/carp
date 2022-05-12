import axios from "axios";
import type { AxiosError, AxiosResponse } from "axios";
import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import type { ErrorShape } from "@dcspark/carp-client/shared/errors";
import type { EndpointTypes } from "@dcspark/carp-client/shared/routes";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";

const urlBase = "http://localhost:3000";
type BlockLatestQuery = EndpointTypes[Routes.blockLatest];

async function query(
  data: BlockLatestQuery["input"]
): Promise<BlockLatestQuery["response"]> {
  const result = await axios.post<
    BlockLatestQuery["response"],
    AxiosResponse<BlockLatestQuery["response"]>,
    BlockLatestQuery["input"]
  >(`${urlBase}/${Routes.blockLatest}`, data);
  return result.data;
}

function getErrorResponse(
  err: AxiosError<ErrorShape, unknown>
): AxiosResponse<ErrorShape, unknown> {
  if (err.response == null) throw new Error(`Unexpected null response`);
  return err.response;
}

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.addressUsed}`, function () {
  this.timeout(10000);

  it("should find the latest block possibly with an offset", async function () {
    const result = await query({
      offset: 0,
    });
    const resultOffByOne = await query({
      offset: 1,
    });
    expect(result.block.height).be.greaterThan(1);
    expect(result.block.height).be.greaterThan(resultOffByOne.block.height);
  });

  it("should error if the offset is too large", async function () {
    try {
      await query({
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
