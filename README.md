# Oura Postgres Sink

**status**: in development

Sync a postgres database with the cardano blockchain using [Oura](https://github.com/txpipe/oura)

**Note**: This project is primarily made for post-Shelley era queries. Although Byron-era transactions will be recorded in the DB, the Typescript API is not particularly written to handle Byron-era wallet management.

## Usage

### Running

#### Setup DB

This repo, by default, is setup to work with a local node. If you want to run on a remote node, you can change the socket in `.env`. If this remote node requires a TCP connection, you can change the `BearerKind` to TCP in the code.

Note: steps assume mainnet

1) `sudo -u postgres createdb oura_postgres_mainnet`
1) `sudo -u postgres createuser oura`
1) `sudo -u postgres psql`
1) `\password oura`
1) Add your username & password to `secrets/db-password` and `secrets/db-user`
1) `\q`
1) Modify the env variables in `.env` if needed (ex: connecting to local node instead of remote)
1) Run the env file (`set -a; . .env; set +a`) 
1) `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)
1) `cargo run`

### Migrations

There is an alias configured for convenience.

- `cargo migrate up`
- `cargo migrate down`
- `cargo migrate help`

## Crates

- **entity**
  - contains the SeaORM models
- **migration**
  - contains the database migrations
- **src**
  - contains the application code
