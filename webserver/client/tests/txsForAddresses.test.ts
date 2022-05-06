import axios from "axios";
import type { AxiosError, AxiosResponse } from "axios";
import { expect } from "chai";
import { Errors } from "../../shared/errors";
import type { ErrorShape } from "../../shared/errors";
import type { EndpointTypes } from "../../shared/routes";
import { Routes } from "../../shared/routes";
import { StatusCodes } from "http-status-codes";
import sortBy from "lodash/sortBy";
import { bech32 } from "bech32";
import Cip5 from "@dcspark/cip5-js";
import { RelationFilterType } from "../../shared/models/common";

const urlBase = "http://localhost:3000";
type HistoryQuery = EndpointTypes[Routes.txsForAddresses];

const hashForUntilBlock =
  "5fc6a3d84cbd3a1fab3d0f1228e0e788a1ba28f682a3a2ea7b2d49ad99645a2c";

async function query(
  data: HistoryQuery["input"]
): Promise<HistoryQuery["response"]> {
  const result = await axios.post<
    HistoryQuery["response"],
    AxiosResponse<HistoryQuery["response"]>,
    HistoryQuery["input"]
  >(`${urlBase}/${Routes.txsForAddresses}`, data);
  return result.data;
}

function getErrorResponse(
  err: AxiosError<ErrorShape, unknown>
): AxiosResponse<ErrorShape, unknown> {
  if (err.response == null) throw new Error(`Unexpected null response`);
  return err.response;
}

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.txsForAddresses}`, function () {
  this.timeout(10000);
  it("should return empty if addresses do not exist", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsfYMUNRxtQ5NNKbWVw3ZJBNcMLLZSoqmD5trHHPBDwsjonoBgw1K6e8Qi8bEMs5Y62yZfReEVSFFMncFYDUHUTMM436KjQ",
        "DdzFFzCqrht4s7speawymCPkm9waYHFSv2zwxhmFqHHQK5FDFt7fd9EBVvm64CrELzxaRGMcygh3gnBrXCtJzzodvzJqVR8VTZqW4rKJ",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.transactions).be.empty;
  });

  it("should return empty if there are no tx after the given address", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsqW5ZTDVX3sR9eEuBr5uPvWoBGaT5GjBQuA2gFL8aRvnecCr73xBsjWnSsebgHAFxEczaUDgW3pMs9Yx4CedeBemyqa1Rr",
      ],
      untilBlock: hashForUntilBlock,
      after: {
        tx: "a5fb58900cbd0a6f5b77bac47fa950555dddb85f684a074b7a748f5b6e3b1aad",
        block:
          "6575c26f4eb1533d2087e5e755ff0b606f4fc663a40f7aa558c38c389400f2f0",
      },
    });
    expect(result.transactions).be.empty;
  });

  it("should throw reference errors for a until block that doesn't exist.", async function () {
    try {
      await query({
        addresses: [
          "Ae2tdPwUPEZHu3NZa6kCwet2msq4xrBXKHBDvogFKwMsF18Jca8JHLRBas7",
        ],
        untilBlock:
          "0000000000000000000000000000000000000000000000000000000000000000",
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.PRECONDITION_REQUIRED);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.UntilBlockNotFound.prefix)
      );
    }
  });

  it("should throw reference errors for a tx that doesn't exist.", async function () {
    try {
      await query({
        addresses: [
          "Ae2tdPwUPEZHu3NZa6kCwet2msq4xrBXKHBDvogFKwMsF18Jca8JHLRBas7",
        ],
        untilBlock: hashForUntilBlock,
        after: {
          tx: "0000000000000000000000000000000000000000000000000000000000000000",
          block:
            "790eb4d6ef2fea7cceebf22c66c20518616d5331966f6f9b4ca3a308b9c3ceb1",
        },
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.PRECONDITION_REQUIRED);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.PageStartNotFound.prefix)
      );
    }
  });

  it("should throw reference errors for a tx that doesn't match the block in after.", async function () {
    try {
      await query({
        addresses: [
          "Ae2tdPwUPEZHu3NZa6kCwet2msq4xrBXKHBDvogFKwMsF18Jca8JHLRBas7",
        ],
        untilBlock: hashForUntilBlock,
        after: {
          tx: "9f93abce0b293b01f62ce9c8b0257a3da8aef27de164a609c32c92dc0a04f58e",
          block:
            "0000000000000000000000000000000000000000000000000000000000000000",
        },
      });
      expect(1).to.be.equal(0); // equivalent to asset false
    } catch (err: any /* eslint-disable-line @typescript-eslint/no-explicit-any */) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const response = getErrorResponse(err);
      expect(response.status).to.be.equal(StatusCodes.PRECONDITION_REQUIRED);
      expect(response.data.reason).to.satisfy((msg: string) =>
        msg.startsWith(Errors.PageStartNotFound.prefix)
      );
    }
  });

  it("should not include a transaction twice if an address appears in both the input & the output", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrht3UrnL3bCK5QMi9XtmkqGG3G2tmuY17tWyhq63S7EzMpJPogoPKx15drcnJkH4A7QdqYgs4h3XD1zXb3zkDyBuAZcaqYDS",
      ],
      untilBlock:
        "bb0c2160852adee50d3dd0ed6c3b5b7ac406a0b51704de3c90f6e5b8563ff69d",
      after: {
        tx: "fc6c0965767fad75baa4ccb9edc35bcd16fa96f7717771699aaa6f8e1ca0dfaa",
        block:
          "bb0c2160852adee50d3dd0ed6c3b5b7ac406a0b51704de3c90f6e5b8563ff69d",
      },
    });
    expect(result.transactions).to.have.length(1);
  });

  it("should return elements sorted asc block height", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrht6pqNhrJwDYh8gchg1h45C2bJRTFKmQbsv1T1EX63kpWtrwYVPTAAmpt29jYoTGBZSTDJfjA3w54kCMmsjKvsnGjnAraoB",
      ],
      untilBlock: hashForUntilBlock,
      after: {
        // 9 months before hashForUntilBlock on mainnet
        tx: "9b79b090c99371da500abb092637d65da2872a7540b025d02bf1240171ec5984",
        block:
          "b687efdd818816cf46ffc65cccb4326c8fc0d64ff2889f808463d8a5ad7819ce",
      },
    });
    expect(result.transactions).to.have.length(2);
    expect(result.transactions[0].block.height).to.be.lessThan(
      result.transactions[1].block.height
    );
  });

  it("should return history for input and output addresses", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
        "DdzFFzCqrht33HAPd4PyqRAhmry5gsSgvZjh8dWdZPuHYchXPbP1W3Rw5A2zwgftbeU9rMu3znnpNib3oFGkmBy3LL8i8VTZhNG9qnwN",
        "DdzFFzCqrhsgXjCq9Gc3RbGkGNnShyMqKcXzvJM4ByLhuPQ77UGRjy59TQbtLdMuJJz9PcFACi5mYrfA9h11vUehcZPCzJUsC7nirrJB",
        "DdzFFzCqrhsksJxdqiVRGY5kZbzKJmMW9qKcZMVZ95oYaDrCHEEk1fxV4QbkoNDu24WY1ZKCUnuizc8SWaVPkEwv66eTtUdsyVRBkgD7",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.transactions.map((tx) => tx.transaction.hash)).to.eql([
      "ee01627b2bfa5bd5a7dd9d2be7f9108ea0c0585c58216cb16d07803ae769b34f",
      "d5dff06dda8659afb095482b95c1f5bf0beba6e2a93f614532769a4a5a575793",
      "b77647288bf6553726c93faa94685637ac2976c85bba3f3dfd4d011de24371cb",
      "62eb7d469b75b0f15c4b9e905346f5dac13ffa97539bb17cb3867ca74aa25a20",
      "753b8b70901835b6d0104df0622ab14e0596b112c49ada022fc2f8210ac73f1b",
      "58e2be124e42ee00f586a603758c5a19de4035cbafd9292e91ff2479b417b975",
      "669055ba7da5d9d372f226219d151947cddf39a28c15339687db2bbd328ef8e0",
      "c47689f318522f0e3059bd95a0373a32bea49a5ce1f77342becd40dba89b9eac",
      "ed74cca21c7a9d9704409dede6ec3f694b53690d800cd545b7322a7fc134cf73",
      "f4d277d925217cd7ad8f17aefd1c389d25bb16ab9f03f0756ae0dea81d29fcad",
      "261a2dcabb4ea95559eae2ed6e7ec33af173c638a0ef0b96cfd177ba98ed3549",
      "1fa195a2ae860eb446eb431f8aece23ad08ca858eed0634fee10c303a9a9c9c1",
      "ccea6f32d0a6145efb07839f82fe6dc1f2adc0395b6806c84165828c3b01c416",
      "523085191ef5f10cda710cad4a5b31b9efea8679ce513a37e32d78393a9644df",
      "7f1bb375346fb5c957a4da190459590774da3fbeb0155c3a2d78f4656762a396",
      "08a863c1862e9847c3e6925a9ecf0af1867bae86dc026f967ed58e602e051c7b",
      "e62b55540020502cdf9033448a2f874cb1d8663fea5420b257d2b6ae4ce2a678",
    ]);
  });

  it("should allow limiting the number of transactions returned", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
        "DdzFFzCqrht33HAPd4PyqRAhmry5gsSgvZjh8dWdZPuHYchXPbP1W3Rw5A2zwgftbeU9rMu3znnpNib3oFGkmBy3LL8i8VTZhNG9qnwN",
        "DdzFFzCqrhsgXjCq9Gc3RbGkGNnShyMqKcXzvJM4ByLhuPQ77UGRjy59TQbtLdMuJJz9PcFACi5mYrfA9h11vUehcZPCzJUsC7nirrJB",
        "DdzFFzCqrhsksJxdqiVRGY5kZbzKJmMW9qKcZMVZ95oYaDrCHEEk1fxV4QbkoNDu24WY1ZKCUnuizc8SWaVPkEwv66eTtUdsyVRBkgD7",
      ],
      untilBlock: hashForUntilBlock,
      limit: 5,
    });
    expect(result.transactions.map((tx) => tx.transaction.hash)).to.eql([
      "ee01627b2bfa5bd5a7dd9d2be7f9108ea0c0585c58216cb16d07803ae769b34f",
      "d5dff06dda8659afb095482b95c1f5bf0beba6e2a93f614532769a4a5a575793",
      "b77647288bf6553726c93faa94685637ac2976c85bba3f3dfd4d011de24371cb",
      "62eb7d469b75b0f15c4b9e905346f5dac13ffa97539bb17cb3867ca74aa25a20",
      "753b8b70901835b6d0104df0622ab14e0596b112c49ada022fc2f8210ac73f1b",
    ]);
  });

  it("should do same history even if addresses sent twice", async function () {
    const resultUnique = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
      ],
      untilBlock: hashForUntilBlock,
    });
    const resultDuplicate = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(resultUnique.transactions).to.eql(resultDuplicate.transactions);
  });

  it("Pagination mid-block should be supported", async function () {
    const result = await query({
      addresses: [
        "addr1q84shx6jr9s258r9m45ujeyde7u4z7tthkedezjm5kdr4um64gv6jqqncjd205c540fgu5450tzvu27n9lk8ulm3s99spva2ru",
      ],
      // make sure if we as for after txIndex 0, txIndex 1 is included in the response
      // AKA support pagination mid-block
      after: {
        tx: "f07d7d5cb0126da7da9f6a067aee00fd42efae94891a42544abfd1759248019d",
        block:
          "728ceadf2d949281591175a6d1641f10f2307eff80eaf59c5300dbd4a5f83554",
      },
      // make sure untilBlock is inclusive
      untilBlock:
        "728ceadf2d949281591175a6d1641f10f2307eff80eaf59c5300dbd4a5f83554",
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "00d6d64b251514c48a9ad75940c5e7031bae5f0d002e9be7f6caf4cc1a78b57f"
    );
  });

  it("Transaction-era transactions should be marked properly", async function () {
    // Byron era
    {
      const result = await query({
        addresses: [
          "Ae2tdPwUPEZLs4HtbuNey7tK4hTKrwNwYtGqp7bDfCy2WdR3P6735W5Yfpe",
        ],
        after: {
          tx: "aef8aa952a11b1225f1c067824f38e0c4b6d478900db6b57f6503b81fbc09427",
          block:
            "07d8aee8a94c6a65b0a6dac7bb43e7f8ddb7320d3c7770db8b1be4fbd685c0aa",
        },
        untilBlock:
          "187c5137b0c2660ad8277c843ddec0deede6da5c2ba50ac8b958127c328ddbee",
      });
      expect(result.transactions).to.have.lengthOf(1);
      expect(result.transactions[0].transaction.hash).to.equal(
        "130f9c6f3dcb0af0733757b301c877ec63d5c127373e75268e8b20c09fa645df"
      );
      expect(result.transactions[0].block.era).to.equal(0);
    }
    // Shelley era
    {
      const result = await query({
        addresses: [
          "addr1q9ya8v4pe33nlkgftyd70nhhp407pvnjjcsddhf64sh9gegwtvyxm7r69gx9cwvtg82p87zpwmzj0kj7tjmyraze3pzqe6zxzv",
        ],
        untilBlock:
          "e99b06115fc0cd221671b686b6d9ef1c6dc047abed2c4f7d3ae528427a746f60",
      });
      expect(result.transactions).to.have.lengthOf(1);
      expect(result.transactions[0].transaction.hash).to.equal(
        "871b14fbe5abb6cacc63f922187c4f10ea9499055a972eb5d3d4e8771af643df"
      );
      expect(result.transactions[0].block.era).to.equal(1);
    }
  });

  it("untilBlock should limit the response", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
        "DdzFFzCqrht33HAPd4PyqRAhmry5gsSgvZjh8dWdZPuHYchXPbP1W3Rw5A2zwgftbeU9rMu3znnpNib3oFGkmBy3LL8i8VTZhNG9qnwN",
        "DdzFFzCqrhsgXjCq9Gc3RbGkGNnShyMqKcXzvJM4ByLhuPQ77UGRjy59TQbtLdMuJJz9PcFACi5mYrfA9h11vUehcZPCzJUsC7nirrJB",
        "DdzFFzCqrhsksJxdqiVRGY5kZbzKJmMW9qKcZMVZ95oYaDrCHEEk1fxV4QbkoNDu24WY1ZKCUnuizc8SWaVPkEwv66eTtUdsyVRBkgD7",
      ],
      untilBlock:
        "4f4b3aaa45ce53a3c3f4c36907f8b4f6ae3e29c7abef567d20b521ee14d70953",
    });
    const last = result.transactions[result.transactions.length - 1];
    expect(last.block.height).to.be.eql(4105397);
  });

  it("Response should have the right shape", async function () {
    const result = await query({
      addresses: [
        "DdzFFzCqrhsnUbJho1ERJsuZxkevYTofBFMuQo5Uaxmb2dHUQX7TzK4C9gN5Yc5Hc4ok4o4wj1krZrgvQWGfd4BgpYFRQUQBgLzZxFi6",
      ],
      untilBlock:
        "6a77b6dc5cfac00c4ca9b1255d0629e5943272f6abce477853a5a088a1093783",
    });
    expect(result.transactions[0]).to.be.eql({
      block: {
        height: 4105739,
        hash: "6a77b6dc5cfac00c4ca9b1255d0629e5943272f6abce477853a5a088a1093783",
        epoch: 190,
        slot: 4107844,
        era: 0,
        tx_ordinal: 0,
        is_valid: true,
      },
      transaction: {
        hash: "ccea6f32d0a6145efb07839f82fe6dc1f2adc0395b6806c84165828c3b01c416",
        payload:
          "82839f8200d8185824825820fd20a38d26386c335851d34569f4f71369b7c5e1cfcac004a553c1bded3f709000ff9f8282d818582183581cd012854ef1ef0b0d5103abf288ba0056209ccdf329114e5cd0f160bea0001a679cbcc41a05f558348282d818584283581c4277f7ecb0809f5096af9eb44b7aa4e20e7fdee892602e2a81c3cd2ea101581e581cfab568307aa0af43ccdbba7988b9312246edb4892c820ed2ddcb412e001a5e0286bf1b00000033488d452effa0818200d818588582584081b3af96ef5b634120542846f983734e95f33425f5890a9ea0c0897c57999464eea7f6a2758553375672c26bda3b5a60e76bc8e6436fc4eed6a741d4e52638325840f94fced2d0c6bc2599072ae20122c3a81ade63537245d8a405e50459771a8a380b59f591c91c10d3a4c048da1410bf4d6219489d810343e6820e01a22d0db00e",
      },
    });
  });

  it("order of tx objects should be by block_num asc, tx_ordinal as", async function () {
    const result = await query({
      addresses: [
        "Ae2tdPwUPEYynjShTL8D2L2GGggTH3AGtMteb7r65oLar1vzZ4JPfxob4b8",
      ],
      untilBlock: hashForUntilBlock,
    });
    const mergedTxs = sortBy(result.transactions, [
      (tx) => tx.block.height,
      (tx) => tx.block.tx_ordinal,
    ]);
    expect(result.transactions).to.be.eql(mergedTxs);
  });

  it("Get payment key that only occurs in input", async function () {
    const result = await query({
      addresses: [
        bech32.encode(
          Cip5.hashes.addr_vkh,
          bech32.toWords(
            Buffer.from(
              "211c082781577c6b8a4832d29011baab323947e59fbd6ec8995b6c5a",
              "hex"
            )
          )
        ),
      ],
      after: {
        block:
          "b51b1605cc27b0be3a1ab07dfcc2ceb0b0da5e8ab5d0cb944c16366edba92e83",
        tx: "79acf08126546b68d0464417af9530473b8c56c63b2a937bf6451e96e55cb96a",
      },
      untilBlock:
        "f0d4b1eed671770194a223eaba3fc0cb0b6787d83c432ec5c24b83620c9b7474",
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "92bdc4f35fd9b363a4eac47898148fd1816efd4260d71e8251ca80dbb7a39ca3"
    );
  });

  it("Get payment key that only occurs in output", async function () {
    const result = await query({
      addresses: [
        bech32.encode(
          Cip5.hashes.addr_vkh,
          bech32.toWords(
            Buffer.from(
              "85abf3eca55024aa1c22b944599b5e890ec12dfb19941229da4ba293",
              "hex"
            )
          )
        ),
      ],
      untilBlock:
        "094ae9802b7e0a8cee97e88cc14a3029f8788d9cb9568ae32337e6ba2c0c1a5b",
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "7e758ee91595c5a7c668fbe41aacc16ed0f27f317db2e70479a8f16ac85ebd6a"
    );
  });

  it("Get tx using base address bech32", async function () {
    const result = await query({
      addresses: [
        "addr1qxz6hulv54gzf2suy2u5gkvmt6ysasfdlvvegy3fmf969y7r3y3kdut55a40jff00qmg74686vz44v6k363md06qkq0q8eqdws",
      ],
      untilBlock:
        "094ae9802b7e0a8cee97e88cc14a3029f8788d9cb9568ae32337e6ba2c0c1a5b",
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "7e758ee91595c5a7c668fbe41aacc16ed0f27f317db2e70479a8f16ac85ebd6a"
    );
  });

  it("Get tx only related by its staking key", async function () {
    const result = await query({
      addresses: [
        "stake1uydrhuvnrhlzpkzrkukp3h4n0fp5dzqzcz36t5thkmfezyc47wa2x",
      ],
      after: {
        tx: "193c753a090fa0e7248590d407137e9439622e2fe818688186aeb47ac9b58fa4",
        block:
          "42b95a9ce5b17f02aa00f99c3bca0a9eccbdbe0942fa246b5376af66979c8c0c",
      },
      untilBlock:
        "d62a740622c27a501697c90fdbdba7ff4931a1ff2ffccdbb5c85bdaa2bec9539",
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "e2da505cca54744a512cccb714bdb71439a28df0b5a122f489ebea4f1c690995"
    );
  });

  it("Gets tx only related by its staking key with explicit filter", async function () {
    const result = await query({
      addresses: [
        "stake1uydrhuvnrhlzpkzrkukp3h4n0fp5dzqzcz36t5thkmfezyc47wa2x",
      ],
      after: {
        tx: "193c753a090fa0e7248590d407137e9439622e2fe818688186aeb47ac9b58fa4",
        block:
          "42b95a9ce5b17f02aa00f99c3bca0a9eccbdbe0942fa246b5376af66979c8c0c",
      },
      untilBlock:
        "d62a740622c27a501697c90fdbdba7ff4931a1ff2ffccdbb5c85bdaa2bec9539",
      relationFilter: RelationFilterType.Withdrawal,
    });
    expect(result.transactions).to.have.lengthOf(1);
    expect(result.transactions[0].transaction.hash).to.equal(
      "e2da505cca54744a512cccb714bdb71439a28df0b5a122f489ebea4f1c690995"
    );
  });

  it("Fail to find tx only related by its staking key with strict filter", async function () {
    const result = await query({
      addresses: [
        "stake1uydrhuvnrhlzpkzrkukp3h4n0fp5dzqzcz36t5thkmfezyc47wa2x",
      ],
      after: {
        tx: "193c753a090fa0e7248590d407137e9439622e2fe818688186aeb47ac9b58fa4",
        block:
          "42b95a9ce5b17f02aa00f99c3bca0a9eccbdbe0942fa246b5376af66979c8c0c",
      },
      untilBlock:
        "d62a740622c27a501697c90fdbdba7ff4931a1ff2ffccdbb5c85bdaa2bec9539",
      relationFilter: RelationFilterType.FILTER_ALL,
    });
    expect(result.transactions).to.have.lengthOf(0);
  });

  it("Get tx using script hash", async function () {
    const result = await query({
      addresses: [
        // note: this is the jpg.store contract address
        // so this is also a test one of the largest joins you could end up doing
        bech32.encode(
          Cip5.hashes.script,
          bech32.toWords(
            Buffer.from(
              "4a59ebd93ea53d1bbf7f82232c7b012700a0cf4bb78d879dabb1a20a",
              "hex"
            )
          ),
          1000
        ),
      ],
      untilBlock:
        "34b1926c6adb2a9b196701e99de1cbd41953a25033bac10d0ae259ea83bb65d2",
    });
    expect(result.transactions).to.have.lengthOf(1);
  });

  it("Two bech32 addresses in the same tx don't result in the tx being duplicated", async function () {
    const result = await query({
      addresses: [
        "addr1q95ldat52vp36p9awxsge5lnre7kr0af8yph7rj50h59pcm85l8qxaqez4dsa7xqdmr83cmzsg2xl4cz8yfljexuj3cqgf6zhz",
        "addr1qx967jynr3gc0n2elhj48a88mghp524fyqhvdenczh9nlzsra34p9pswlrq86nq63hna7p4vkrcrxznqslkta9eqs2nsr793vd",
      ],
      untilBlock:
        "a8269def34ff4dcb9801934e8a6ea22ed081a1991c5900282c9236a04cff5c3d",
    });

    const deduplicated = new Set();
    const duplicated = new Set();
    for (const tx of result.transactions) {
      if (deduplicated.has(tx.transaction.hash)) {
        duplicated.add(tx.transaction.hash);
      } else {
        deduplicated.add(tx.transaction.hash);
      }
    }
    expect(Array.from(duplicated)).to.be.eql([]);
  });

  it("Two credentials in the same tx don't result in the tx being duplicated", async function () {
    const result = await query({
      addresses: [
        "8200581c69f6f57453031d04bd71a08cd3f31e7d61bfa939037f0e547de850e3",
        "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
      ],
      untilBlock:
        "a8269def34ff4dcb9801934e8a6ea22ed081a1991c5900282c9236a04cff5c3d",
    });

    const deduplicated = new Set();
    const duplicated = new Set();
    for (const tx of result.transactions) {
      if (deduplicated.has(tx.transaction.hash)) {
        duplicated.add(tx.transaction.hash);
      } else {
        deduplicated.add(tx.transaction.hash);
      }
    }
    expect(Array.from(duplicated)).to.be.eql([]);
  });
});
