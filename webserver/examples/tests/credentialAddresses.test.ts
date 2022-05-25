import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";
import cml from "@dcspark/cardano-multiplatform-lib-nodejs";
import { CREDENTIAL_LIMIT } from "@dcspark/carp-client/shared/constants";

const urlBase = "http://localhost:3000";

const hashForUntilBlock =
  "4de6fcf07767a2d47d1b8e06a1396694adf4332b77f70574a2d4475d11633ffe";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.credentialAddress}`, function () {
  this.timeout(100_000);

  // it("should throw on invalid address", async function () {
  //   try {
  //     await query(urlBase, Routes.credentialAddress, {
  //       credentials: [
  //         "DdzFFzCqrht4wFnWC5TJJC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
  //       ],
  //       untilBlock: hashForUntilBlock,
  //     });
  //     expect(1).to.be.equal(0); // equivalent to asset false
  //   } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
  //     // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
  //     const response = getErrorResponse(err);
  //     expect(response.status).to.be.equal(StatusCodes.UNPROCESSABLE_ENTITY);
  //     expect(response.data.reason).to.satisfy((msg: string) =>
  //       msg.startsWith(Errors.IncorrectAddressFormat.prefix)
  //     );
  //   }
  // });

  // it("should return empty if addresses do not exist", async function () {
  //   const result = await query(urlBase, Routes.credentialAddress, {
  //     credentials: [
  //       "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8b",
  //     ],
  //     untilBlock: hashForUntilBlock,
  //   });
  //   expect(result.addresses).be.empty;
  // });

  // it("should not include an address twice if the credential appears twice in an address", async function () {
  //   const result = await query(urlBase, Routes.credentialAddress, {
  //     credentials: [
  //       "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
  //       "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
  //     ],
  //     untilBlock: hashForUntilBlock,
  //   });
  //   expect(result.addresses).to.be.eql([
  //     "addr1qx967jynr3gc0n2elhj48a88mghp524fyqhvdenczh9nlzsra34p9pswlrq86nq63hna7p4vkrcrxznqslkta9eqs2nsr793vd",
  //   ]);
  // });

  // it("returns the expected pagination result", async function () {
  //   const credentials = [
  //     "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
  //   ];
  //   const result = await query(urlBase, Routes.credentialAddress, {
  //     credentials,
  //     untilBlock: hashForUntilBlock,
  //   });

  //   expect(result.pageInfo).to.be.eql({
  //     endCursor: {
  //       address:
  //         "addr1qx967jynr3gc0n2elhj48a88mghp524fyqhvdenczh9nlzsra34p9pswlrq86nq63hna7p4vkrcrxznqslkta9eqs2nsr793vd",
  //       block:
  //         "b67193995969136de07a996122caf3c646c7661fa1c084b696c41962cbeec062",
  //       tx: "b67193995969136de07a996122caf3c646c7661fa1c084b696c41962cbeec062",
  //     },
  //     hasNextPage: false,
  //   });
  // });

  it("paginates correctly", async function () {
    // const credential = cml.StakeCredential.from_bytes(
    //   Buffer.from(
    //     "8200581c3d35de9ece98ddb4773ab33880d28202bc50cf0accbaf6d06eb03722",
    //     "hex"
    //   )
    // );
    const credentials = [
      "8200581c3d35de9ece98ddb4773ab33880d28202bc50cf0accbaf6d06eb03722",
    ];
    const resultPageOne = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock: hashForUntilBlock,
    });

    expect(resultPageOne.addresses).to.have.length(CREDENTIAL_LIMIT.RESPONSE);
    const resultPageTwo = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock: hashForUntilBlock,
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      after: resultPageOne.pageInfo.endCursor!,
    });
    // expect(result.pageInfo).to.be.eql({
    //   endCursor: {
    //     address:
    //       "addr1qx967jynr3gc0n2elhj48a88mghp524fyqhvdenczh9nlzsra34p9pswlrq86nq63hna7p4vkrcrxznqslkta9eqs2nsr793vd",
    //     block:
    //       "b67193995969136de07a996122caf3c646c7661fa1c084b696c41962cbeec062",
    //     tx: "b67193995969136de07a996122caf3c646c7661fa1c084b696c41962cbeec062",
    //   },
    //   hasNextPage: false,
    // });
  });
});
