---
sidebar_position: 1
---

# Carp (Cardano Postges Indexer)

Syncs Cardano blockchain information a Postgres database.

- Backend: written in Rust using [Oura](https://github.com/txpipe/oura) and [CML](https://github.com/dcSpark/cardano-multiplatform-lib).
- Sever & Client: Written using Typescript

# Core pillars

- **Speed**: queries should be fast so they can be used inside production applications like wallets without the user feeling the application is not responsive.
- **Modular**: it should be easy to enable only the database functionality you need to keep sync times fast and size requirements low.
- **Flexible**: instead of assuming the format users will need, prefer to use raw cbor or raw bytes. Almost all applications already implement [CML](https://github.com/dcSpark/cardano-multiplatform-lib) so they should be able to parse this data without any issue.
- **Type safe**: using and exposing types to avoid bugs is crucial in financial software. Database queries and the web server should have all its types checked and available to users.
- **Documented**: Although we have to assume the user has read the [cardano ledger specs](https://github.com/input-output-hk/cardano-ledger), design decisions, usage and pitfalls should all be documented.

# FAQ

Q) How does it take to sync the database from scratch?<br />
A) Around 4-5 days. The first epochs are really fast, but Alonzo takes about ~1hr per epoch. We recommend tacking occasional snapshots of the database so that you can easily spin up new nodes or recover from crashes without having to resync from scratch.

Q) How long to query history?<br />
A) Querying the transaction history for an address should a <10ms for local queries (no network overhead). Of course, it will take longer if you're using a slow machine or if your machine is at max utilization.

Q) How to launch my own network?<br />
A) TBD. We support parsing genesis blocks so it should be doable. Feel free to make a PR for more concrete steps.

Q) What are the risks and common pitfalls of using this project?<br />
A) See [pitfalls](./pitfalls)

# Running

This project contains two different parts to it:

1. An in [indexer README](./indexer/intro.md) which stores the chain to a Postgres database
2. A [webserver README](./webserver/intro.md) which provides a REST API for querying the database (ex: for light wallets)
