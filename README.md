# Carp - Cardano Postgres Indexer

**Important**:
- Documentation: [here](https://dcspark.github.io/carp/docs/intro)
- NPM package (client): [here](https://www.npmjs.com/package/@dcspark/carp-client)

Welcome to Carp, a Cardano Postgres Indexer. This project is designed to sync data from the Cardano blockchain and store it into a Postgres database. The backend is written in Rust using Oura and CML, while the server and client are written in TypeScript.

## Core Pillars
The core pillars of Carp are:

**Speed**: Queries should be fast so they can be used inside production applications like wallets without the user feeling the application is not responsive.

**Modular**: It should be easy to enable only the database functionality you need to keep sync times fast and size requirements low.

**Flexible**: Instead of assuming the format users will need, we prefer to use raw cbor or raw bytes. Almost all applications already implement CML so they should be able to parse this data without any issue.

**Type safe**: Using and exposing types to avoid bugs is crucial in financial software. Database queries and the web server should have all its types checked and available to users.

**Documented**: Although we have to assume the user has read the Cardano ledger specs, design decisions, usage, and pitfalls should all be documented​1.

## FAQ

**How long does it take to sync the database from scratch?**

Around 4-5 days. The first epochs are really fast, but Alonzo takes about ~1hr per epoch. We recommend taking occasional snapshots of the database so that you can easily spin up new nodes or recover from crashes without having to resync from scratch.

**How long does it take to query history?**

Querying the transaction history for an address should take <10ms for local queries (no network overhead). Of course, it will take longer if you're using a slow machine or if your machine is at max utilization.

**How can I launch my own network?**

We support parsing genesis blocks so it should be doable. However, this feature is still in development. Feel free to make a PR for more concrete steps​1​.

## Running Carp
This project contains two different parts:

- An indexer that stores the chain to a Postgres database.
- A webserver that provides a REST API for querying the database (ex: for light wallets)​1​.

For detailed instructions on how to run each component, please refer to the respective README files of the indexer and the webserver.

## Contributing

Contributions to Carp are very welcome. If you're interested in improving the project, feel free to make a PR.
