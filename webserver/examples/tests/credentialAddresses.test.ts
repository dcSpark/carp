import { expect } from "chai";
import { Errors } from "@dcspark/carp-client/shared/errors";
import { Routes } from "@dcspark/carp-client/shared/routes";
import { StatusCodes } from "http-status-codes";
import { query, getErrorResponse } from "@dcspark/carp-client/client/src/index";
import { CREDENTIAL_LIMIT } from "@dcspark/carp-client/shared/constants";

const urlBase = "http://localhost:3000";

const hashForUntilBlock =
  "4de6fcf07767a2d47d1b8e06a1396694adf4332b77f70574a2d4475d11633ffe";

// eslint-disable-next-line mocha/no-setup-in-describe
describe(`/${Routes.credentialAddress}`, function () {
  this.timeout(100_000);

  it("should throw on invalid address", async function () {
    try {
      await query(urlBase, Routes.credentialAddress, {
        credentials: [
          "DdzFFzCqrht4wFnWC5TJJC9xZWq589iKyCrWa6hek3KKevyaXzQt6FsdunbkZGzBFQhwZi1MDpijwRoC7kj1MkEPh2Uu5Ssz",
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
    const result = await query(urlBase, Routes.credentialAddress, {
      credentials: [
        "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8b",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.addresses).be.empty;
  });

  it("should not include an address twice if the credential appears twice in an address", async function () {
    const result = await query(urlBase, Routes.credentialAddress, {
      credentials: [
        "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
        "8200581c8baf48931c5187cd59fde553f4e7da2e1a2aa9202ec6e67815cb3f8a",
      ],
      untilBlock: hashForUntilBlock,
    });
    expect(result.addresses).to.be.eql([
      "addr1qx967jynr3gc0n2elhj48a88mghp524fyqhvdenczh9nlzsra34p9pswlrq86nq63hna7p4vkrcrxznqslkta9eqs2nsr793vd",
    ]);
    expect(result.pageInfo).to.be.eql({
      hasNextPage: false,
    });
  });

  it("paginates correctly", async function () {
    const credentials = [
      "8200581c3d35de9ece98ddb4773ab33880d28202bc50cf0accbaf6d06eb03722",
    ];
    const resultPageOne = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock: hashForUntilBlock,
    });

    expect(resultPageOne.addresses).to.have.length(CREDENTIAL_LIMIT.RESPONSE);
    expect(resultPageOne.pageInfo.hasNextPage).to.be.eql(true);
    const resultPageTwo = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock: hashForUntilBlock,
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      after: resultPageOne.addresses[resultPageOne.addresses.length - 1],
    });
    expect(resultPageTwo.addresses).to.have.length(4);
    expect(resultPageOne.addresses.concat(resultPageTwo.addresses)).to.be.eql([
      "addr1q9u6dqxj5dsm6tkke0l3k06rpl0l7ddmrqfw2gh2uhq4nkfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qa5ct4w",
      "addr1qxa3l5n0xgeql384gvxg260z32nqdwt7y7pgcukujjljpjfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q3p3tpk",
      "addr1qxppvsshgauql2wqpuhqq78wu5f7jdz7a8vuk8gmpyghszfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qfexug9",
      "addr1q9j8fea0z04cu99nnvf5p2h8r6ltqrsyx2gs6gys9aqlecfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qzj3q0n",
      "addr1q8p409pxqyr5x7khhw4zj495yv7ama04adrhzuecypc6ycpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qc45yr5",
      "addr1qxrcfd64jg5jka5ryjxgyusunz3mdrm4apdsh42u32szw5faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q4mcdsh",
      "addr1q8yagqwn06c5s5rvrsajl2h90um2w2ahupj6p4a3zz43v0paxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qdx88py",
      "addr1q9p32xatffun6unxtuhdhja45pwwe5aej24hwrwl4jxgurfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qtp4as6",
      "addr1q82vgy0r7fssqsymy3r8nd9zrql8duqnyne0uljf3zpad0faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q90r7tq",
      "addr1qxu4hu3sdh6kpjsq7whclukcprawr96y6wzlcrwuknmdf33axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qspkv0z",
      "addr1q9mazt6xqln5kfgma32ynwf6e2z8zj9e05ws22v9azrmuveaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qy86hem",
      "addr1q842fg6w7phnsr2qrgklu0lr99628g45m8nyksg037es0ppaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qj04kg9",
      "addr1qyextj63h0vuk53yp65ay69kzj4fvfgk3junsehc9f74273axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qdqggnw",
      "stake1uy7nth57e6vdmdrh82en3qxjsgptc5x0ptxt4aksd6crwgs2mqlk9",
      "addr1qynzmykmtmpwqst07s8jckgdahc74a02zlg4ghj662grsaeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qa3ypr5",
      "addr1q8auekc3xwg7jndu8d53t8vhfwmdldlz24d7xwx4rhqwvneaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q0qc59h",
      "addr1q95d5tvyerq5955fjc9md7q76enzhy5xmzq3t0p5mdslrnfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q2knqzs",
      "addr1qxgnpsnpt9cgtr539nu79h4mjuxtxajl85ap2wza3hg9l9faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3ql4zgyf",
      "addr1q9mdhgaa3h78hg3xzg8gqtpckxwcsz05ndymqr9xz3vlkpeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qxxlcxc",
      "addr1q8luglccqzuetvzkvd9ntpg26zslw5f0qffcgh8pf03pupfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qd7kmdz",
      "addr1qy58nuc4045q3vm3e4qs4hm53s90668emhe8chst6fxsk4faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qzrr9u6",
      "addr1q8eelqgzum0r2r72xnnvn0eqct6huz5s69ksfade8a35refaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qqkh2mg",
      "addr1qx73hkrxfjppvujmlddjvwtt7wfk6kn588k82u7zjy02x5faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q4szm7r",
      "addr1qyak9gmtnltmlx455hxy7wrffmhcr9lfjvjunks5f05hnnpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qjfv0wy",
      "addr1q8kzuj29g8764hpzz7nv6ug8em8q9t949qzwsw46mme25yfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q4f2swl",
      "addr1q96408nhkgv6xxk852g6zzhf42rv3adypjm92lkn7p0glneaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qyy8ae4",
      "addr1qy08z89h0cdcfr0nrasz5s8pqydnepuqxdsgke32adrcaj3axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q3s6alp",
      "addr1qxc5phkwhxtwutgr8umkn7twetlulpslypj3vyze5w7u2l3axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qaxmanx",
      "addr1q88hnylqm2klajm8038eezjm5rl9m33xvvwv93rkrpjmrxeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qdj967u",
      "addr1qyj4z84qhk7rs9yz26ndl0x3ywg88atmzm3pzutjs8m0pqpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qrjn56n",
      "addr1q84q8ujsjxennq8yfeafejwntpf29rsl3nm74ls6mwta37paxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q0l3vt7",
      "addr1q80wued2kfhxhnq4jt8pcsmk29fl9dnke5lj9lgj402wprpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q7s2u8j",
      "addr1qx6jneyf6ec0sgkmgn08lu0l2e6dzg8gv9z55z99e3scygfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3quwd5nm",
      "addr1q8x2cu6fy2ww9u3n36l4czvkrqd8d4an7mgxt8m5qefqlapaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qhqr7xc",
      "addr1qypc5x66gvgmzt7gf7kxfva2nwu5fwsytlh5w9fkyftn6x3axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qkstr3v",
      "addr1q8hln6nq06ptndjl2kyce4clnwm20q83hklx4ds6pn4katpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qfkh832",
      "addr1q8vs2ek4j88nau5pjx93e5mw74h7xkdgj4kcgukcu8kcl5eaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qh6udmq",
      "addr1qxjp780kkdpkcca9w86krdx362m7k4fpv7g5zp3rc2gph53axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qc5u8tk",
      "addr1q990yhj720ja320039tdku5gqlfsj923myr977u4v7hhk93axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qkeat4m",
      "addr1q8qve2pdapnkdqm0ptv0aa2qxsce97jjg0xqkxh9y4a5dxeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qkzskwe",
      "addr1qyz38adhjq8faxm34rcy070cvcjwlgwth86qw6sdgtghqqeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qtput5z",
      "addr1qyh8dv5wvhnpd7egjwtr9l2g483ylpa3wfa0gxyxsllkh2faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q0xxsyu",
      "addr1qymjjdeks4packrqe5n65csarlthgs4a0f75tfls024stupaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qd85eqq",
      "addr1q8xespn45hkvktm5y44sna89v52grga3r70qgdfhnvrhwaeaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q5qnwdg",
      "addr1qxc4fpfgjw5k74xamrlp8makg2vu6g84jshl0eu27mcpv8eaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qnseahh",
      "addr1qxqd9ecg443rvxxvu8m04t5hwvduxell4p5dtmykchjn4spaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q80nzjf",
      "addr1q97pr5d5kv9gk3dhzdtr894270r4tw09f9wsm47a7ew8cmfaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qg35ry3",
      "addr1qyc8jymv33qynmk4pcmwj5w506t4fn8tf92edvx9m74jc9faxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qcdvpru",
      "addr1q92ntqnmd0cd3tdukat38yphk9l0fx3kfxmu03nprdaxkleaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q35q8a4",
      "addr1q85mte3470v403kadnfm0e0nu9ezq6aqjge2j0p2gldn8peaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qlqww09",
      "addr1q88fel0dpqspj7rv275sl7w70gkzr2jgl3re703wpt9ayafaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q3hqle4",
      "addr1q9z8g459ncdlpt8mrdkmf4s6hzx3vkznnxz5akg75dgsu5paxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q65tmh7",
      "addr1qyez30ynkvmue28czdlz94myhe8uzpreu7sxnhw04j550qpaxh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3q2unluy",
      "addr1qyn338gtagxr3wt82etckyhg29mj5r82gtqpu0t82ugcwe3axh0fan5cmk68ww4n8zqd9qszh3gv7zkvhtmdqm4sxu3qqav847",
    ]);
  });

  it("detects page on exact match", async function () {
    const credentials = [
      "8200581c3d35de9ece98ddb4773ab33880d28202bc50cf0accbaf6d06eb03722",
    ];
    const resultPageOne = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock:
        "d4cc291c94652ae5618cea7ddcbec808b902fe18e7baf357ab6938822989c8ef",
    });

    const resultPageTwo = await query(urlBase, Routes.credentialAddress, {
      credentials,
      untilBlock:
        "d4cc291c94652ae5618cea7ddcbec808b902fe18e7baf357ab6938822989c8ef",
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      after: resultPageOne.addresses[resultPageOne.addresses.length - 1],
    });
    expect(resultPageTwo.addresses).to.have.length(1);
  });
});
