---
sidebar_position: 3
---

# Running

## Setting up cardano-node

The indexer, by default, is setup to work with a local node.\*, so you will have to run a synced copy of [cardano-node](https://github.com/input-output-hk/cardano-node/)

\*If you want to run on a remote node, you can change the socket in `.env`. If this remote node requires a TCP connection, you can change the `BearerKind` to TCP in the code.

## Setting up the database

Note: steps assume mainnet

1. `sudo -u postgres createdb carp_mainnet`
1. `sudo -u postgres psql -c 'ALTER DATABASE carp_mainnet SET jit_above_cost = -1;'`
1. `sudo -u postgres createuser carp`
1. `sudo -u postgres psql -c "\password carp"`
1. Add your database name & user password to `secrets/.pgpass`
1. `chmod 600 secrets/.pgpass`
1. Modify the env variables in `.env` if needed (ex: connecting to local node instead of remote)
1. At the root of the indexer folder, run `set -a; . ./.env; set +a` - note you will have to re-run this command every time you reopen your shell
1. `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)

## Running the indexer

`cargo run -- --plan execution_plans/default.toml`

Reminder: you can visualize the execution plan using `cargo plan-visualizer --plan execution_plans/default.toml -o plan-visualizer/out`
