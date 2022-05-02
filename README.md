# Oura Postgres Sink

Sync a postgres database with the cardano blockchain using [Oura](https://github.com/txpipe/oura)

# Core pillars

- Speed
- Modular
- Flexible (return cbor instead of parsed)
- Type safe
- Documented

# FAQ

Q) How long to sync?
A) TBD

Q) How long to query history?
A) TBD

Q) How to launch my own network?
A) TBD

## Database setup

This repo, by default, is setup to work with a local node. If you want to run on a remote node, you can change the socket in `.env`. If this remote node requires a TCP connection, you can change the `BearerKind` to TCP in the code.

Note: steps assume mainnet

1. `sudo -u postgres createdb oura_postgres_mainnet`
1. `sudo -u postgres psql -c 'ALTER DATABASE oura_postgres_mainnet SET jit_above_cost = -1;'`
1. `sudo -u postgres createuser oura`
1. `sudo -u postgres psql -c "\password oura"`
1. Add your database name & user password to `secrets/.pgpass`
1. `chmod 600 secrets/.pgpass`
1. Modify the env variables in `.env` if needed (ex: connecting to local node instead of remote)
1. Run the env file (`set -a; . .env; set +a`) - note you will have to re-run this command every time you reopen your shell
1. `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)

### Running

This project contains two different parts to it:

1. An in [indexer README](./indexer/README.md) which stores the chain to a Postgres database
2. A [webserver README]('./webserver/README.md) which provides a REST API for querying the database (ex: for light wallets)

You may also be interested in:

2. The [generated SQL](./webserver/bin/schema.sql) if you want to run your own queries
