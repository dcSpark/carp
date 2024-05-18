import { expect } from "chai";
import { Errors } from "@dcspark/carp-client";
import { Routes } from "@dcspark/carp-client";
import { StatusCodes } from "http-status-codes";
import { bech32 } from "bech32";
import Cip5 from "@dcspark/cip5-js";
import { query, getErrorResponse } from "@dcspark/carp-client";

const urlBase = "http://localhost:3000";

const hashForUntilBlock =
  "5fc6a3d84cbd3a1fab3d0f1228e0e788a1ba28f682a3a2ea7b2d49ad99645a2c";

const nonExistingAddr =
  "DdzFFzCqrhtBBX4VvncQ6Zxn8UHawaqSB4jf9EELRBuWUT9gZTmCDWCNTVMotEdof1g26qbrDc8qcHZvtntxR4FaBN1iKxQ5ttjZSZoj";
const genesisAddr =
  "Ae2tdPwUPEZ1zsFUP2eYpyRJooGpYSBzR1jZsdK6ioAqR9vUcBiwQgyeRfB";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.addressUsed}`, function () {
  this.timeout(100_000);

  it("should throw on invalid address", async function () {
    try {
      await query(urlBase, Routes.addressUsed, {
        addresses: [
          "Ae2tdPwUPEZHu3NZa6kCwet2msq4xrBXKHBDvogFKwMs8Jca8JHLRBas7",
        ],
        untilBlock: hashForUntilBlock,
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.UNPROCESSABLE_ENTITY);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.IncorrectAddressFormat.prefix)
      );
    }
  });

  it("should return empty if addresses do not exist", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrhsfYMUNRxtQ5NNKbWVw3ZJBNcMLLZSoqmD5trHHPBDwsjonoBgw1K6e8Qi8bEMs5Y62yZfReEVSFFMncFYDUHUTMM436KjQ",
        "DdzFFzCqrht4s7speawymCPkm9waYHFSv2zwxhmFqHHQK5FDFt7fd9EBVvm64CrELzxaRGMcygh3gnBrXCtJzzodvzJqVR8VTZqW4rKJ",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.addresses).be.empty;
  });

  it("filters for legacy addresses", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrht4wFnWC5TJA5UUVE54JC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
        nonExistingAddr,
        "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
        genesisAddr,
        "DdzFFzCqrht2Hw9qp1YcqsMJfwjMXiJR46RXU8KFALErRXnjHnjwBPCP8FDjwgUQkZeGghu69YP71ZU67EDsXa5G3g8D2Kr5XZ7Jc42b",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.addresses).to.be.eql([
      genesisAddr,
      "DdzFFzCqrht2Hw9qp1YcqsMJfwjMXiJR46RXU8KFALErRXnjHnjwBPCP8FDjwgUQkZeGghu69YP71ZU67EDsXa5G3g8D2Kr5XZ7Jc42b",
      "DdzFFzCqrht4wFnWC5TJA5UUVE54JC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
      "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
    ]);
  });

  it("filters with pagination for legacy addresses", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrht4wFnWC5TJA5UUVE54JC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
        nonExistingAddr,
        "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
        genesisAddr,
        "DdzFFzCqrht2Hw9qp1YcqsMJfwjMXiJR46RXU8KFALErRXnjHnjwBPCP8FDjwgUQkZeGghu69YP71ZU67EDsXa5G3g8D2Kr5XZ7Jc42b",
      ],
      untilBlock: hashForUntilBlock,
      after: {
        tx: "46be91680926afd878beb2eab6734d89c60d1326525605e8c59ad29efddc8abc",
        block:
          "99d9b03900855d75346962fe44a4c27749760b5c580610f46ab5b824f17ff9dd",
      },
    });
    expect(result.addresses).to.be.eql([
      "DdzFFzCqrht2Hw9qp1YcqsMJfwjMXiJR46RXU8KFALErRXnjHnjwBPCP8FDjwgUQkZeGghu69YP71ZU67EDsXa5G3g8D2Kr5XZ7Jc42b",
      "DdzFFzCqrht4wFnWC5TJA5UUVE54JC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
      "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
    ]);
  });

  it("should not include an address twice if an address appears multiple times on-chain", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrht3UrnL3bCK5QMi9XtmkqGG3G2tmuY17tWyhq63S7EzMpJPogoPKx15drcnJkH4A7QdqYgs4h3XD1zXb3zkDyBuAZcaqYDS",
      ],
      untilBlock:
        "bb0c2160852adee50d3dd0ed6c3b5b7ac406a0b51704de3c90f6e5b8563ff69d",
    });
    expect(result.addresses).to.have.length(1);
  });

  it("sorts result lexicographically", async function () {
    const addresses = [
      // note: these are not ordered
      "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
      "DdzFFzCqrht4wFnWC5TJA5UUVE54JC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
      "DdzFFzCqrht2Hw9qp1YcqsMJfwjMXiJR46RXU8KFALErRXnjHnjwBPCP8FDjwgUQkZeGghu69YP71ZU67EDsXa5G3g8D2Kr5XZ7Jc42b",
    ];
    const result = await query(urlBase, Routes.addressUsed, {
      addresses,
      untilBlock: hashForUntilBlock,
    });

    addresses.sort();
    expect(result.addresses).to.be.eql(addresses);
  });

  it("should return the same result if an address is sent twice", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
        "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
      ],
      untilBlock: hashForUntilBlock,
    });

    expect(result.addresses).to.be.eql([
      "DdzFFzCqrht62k6YFcieBUwxkq2CLSi4Pdvt3bd6ghq5P7fTgp8n5pRyQK45gN8A2Udyaj9mFRdK1eUoxy1QjcU5AuCix5uJB3zdBgkf",
    ]);
  });

  it("can handle payment keys", async function () {
    const addresses = [
      // addr_vkh1yywqsfup2a7xhzjgxtffqyd64verj3l9n77kajyetdk95nyxlwu
      bech32.encode(
        Cip5.hashes.addr_vkh,
        bech32.toWords(
          Buffer.from(
            "211c082781577c6b8a4832d29011baab323947e59fbd6ec8995b6c5a",
            "hex"
          )
        )
      ),
      // addr_vkh1qqqqsfup2a7xhzjgxtffqyd64verj3l9n77kajyetdk95jgcc59
      bech32.encode(
        Cip5.hashes.addr_vkh,
        bech32.toWords(
          Buffer.from(
            "0000082781577c6b8a4832d29011baab323947e59fbd6ec8995b6c5a",
            "hex"
          )
        )
      ),
    ];
    const result = await query(urlBase, Routes.addressUsed, {
      addresses,
      untilBlock:
        "a8269def34ff4dcb9801934e8a6ea22ed081a1991c5900282c9236a04cff5c3d",
    });

    expect(result.addresses).to.be.eql([addresses[0]]);
  });

  it("is still fast on legacy address with many txs", async function () {
    const result = await query(urlBase, Routes.addressUsed, {
      addresses: [
        "DdzFFzCqrhstmqBkaU98vdHu6PdqjqotmgudToWYEeRmQKDrn4cAgGv9EZKtu1DevLrMA1pdVazufUCK4zhFkUcQZ5Gm88mVHnrwmXvT",
      ],
      untilBlock:
        "bb7507affd7d04eb2e2ba2df8366204b447a15550caefb6d94e601a4df44b641",
    });
    expect(result.addresses).to.be.eql([
      "DdzFFzCqrhstmqBkaU98vdHu6PdqjqotmgudToWYEeRmQKDrn4cAgGv9EZKtu1DevLrMA1pdVazufUCK4zhFkUcQZ5Gm88mVHnrwmXvT",
    ]);
  });
});
