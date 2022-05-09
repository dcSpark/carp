# Common Pitfalls

- During the Byron era, Cardano had a feature where users could add custom metadata into the addresses themselves. The size of this metadata was for some time uncapped, so some addresses that were created on-chain are thousand of characters long (notably, they are larger than the max indexed row size). To tackle this, we truncate these addresses to 500 bytes (which means they won't be parsable as valid Cardano addresses)
- Some on-chain Shelley addresses are incorrect (a similar bug to the one mentioned above)
- Genesis block in general breaks a lot of assumptions you may have. For example
  - Transactions are not ordered
  - Transactions don't contain any transaction body. Instead, the address is the transaction in a sense and the transaction hash is just the has of the address.
- We store StakeCredentials and not key hashes. StakeCredentials are defined in the Cardano CDDL as a hash and a tag the defines what kind of hash it is (ex: key hash or script hash)
- Transactions can fail on-chain in Cardano. Whether or not the transaction was valid is stored in the Transaction table. The TransactionInput table contains the input which was consumed (which depends on whether or not the transaction failed)
- Transaction metadata labels are stored as a byte array because u64 is [not supported in sqlx](https://github.com/launchbadge/sqlx/issues/1374)

# Risks with using this codebase:

- **Missing functionality in binary data parsing libraries**: This codebase parses raw CBOR (binary data) from the Cardano blockchain using two libraries: Pallas and CML. It has happened in the past that one of these libraries is missing some feature of the Cardano blockchain which could cause Carp to fail if these ever appear on-chain
- **Incompatibility bugs between parsing libraries**: This project uses both Pallas and CML. Although both libraries implement some overlapping features, they are occasionally implemented differently. This should not cause any issues, but subtle implementation differences may cause issues leading to bugs in Carp
- **Cardano ledger bugs** (yes, this happens): There has been multiple occasions where the Cardano node itself has a bug in it causing the data generated not to be parsable by Pallas/CML until patched.
- **Byron-era limitations**: CML (at the time of writing) doesn't support most of Byron-era structures. We use Pallas inside Carp so they should appear in the SQL database properly, but they won't be necessarily be parsable if you're reading from the database using CML. There is also some Byron-era features we didn't expose like "Epoch Boundary Blocks" (EBBs)
