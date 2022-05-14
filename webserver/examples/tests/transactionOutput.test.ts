import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";

const urlBase = "http://localhost:3000";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.addressUsed}`, function () {
  this.timeout(10000);

  it("should find outputs given utxo pointers", async function () {
    const result = await query<Routes.transactionOutput>(urlBase, {
      utxoPointers: [
        {
          txHash:
            "7775d5e094b3660cae2464da5ba029134bfa9ca410cc3c7198d23731855bc3d0",
          index: 0,
        },
        {
          txHash:
            "00001781e639bdf53cdac97ebbaf43035b35ce59be9f6e480e7b46dcd5c67028",
          index: 4,
        },
      ],
    });

    expect(result.utxos).to.be.eql([
      {
        txHash:
          "7775d5e094b3660cae2464da5ba029134bfa9ca410cc3c7198d23731855bc3d0",
        index: 0,
        payload:
          "82582b82d818582183581c962e3a277a62aafd441e9d0e98d79be3d25db0aa57feb7daf52777e3a0001a6acabec11a07794402",
      },
      {
        txHash:
          "00001781e639bdf53cdac97ebbaf43035b35ce59be9f6e480e7b46dcd5c67028",
        index: 4,
        payload:
          "82581d6100000000000000000000000000000000000000000000000000000000821a0014851ea1581cdb01dec7311778ad90b72627a38cd6ec61a298f964d2320b4a67c23ba14356495001",
      },
    ]);
  });

  it("should reject invalid tx hashes", async function () {
    try {
      await query<Routes.transactionOutput>(urlBase, {
        utxoPointers: [
          {
            txHash: "777",
            index: 0,
          },
        ],
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.PRECONDITION_REQUIRED);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.IncorrectTxHashFormat.prefix)
      );
    }
  });
});
