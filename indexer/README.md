# oura-postgres-sink Indexer

`cargo run` will start oura-postgres-sink.

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
1. At the root of the repo, run `set -a; . ./.env; set +a` - note you will have to re-run this command every time you reopen your shell
1. `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)

# Run

`cargo run -- --plan execution_plans/default.ini`

Note: you can visualize the execution plan using `cargo plan-visualizer --plan execution_plans/default.ini`

# IDE setup

1. Install `rust-analyzer`
1. Add the following to your vs-code `"rust-analyzer.linkedProjects": ["./indexer/Cargo.toml"]`

### Migrations

There is an alias configured for convenience.

- `cargo migrate up`
- `cargo migrate down`
- `cargo migrate help`

### Rollbacks

Utility to easily rollback the database state

- `cargo rollback era 3 up`
- `cargo rollback epoch 200`
- `cargo rollback height 1000`
