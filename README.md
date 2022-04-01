# Oura Postgres Sink

**status**: in development

Sync a postgres database with the cardano blockchain using [Oura](https://github.com/txpipe/oura)

## Usage

### Running

#### Setup DB

Note: steps assume mainnet

1) `sudo -u postgres createdb oura_postgres_mainnet`
1) `sudo -u postgres createuser oura`
1) `sudo -u postgres psql`
1) `\password oura`
1) Add your username & password to `secrets/db-password` and `secrets/db-user`
1) `\q`
1) Modify the env variables in `.env` if needed (ex: connecting to local node instead of remote)
1) Run the env file (`set -a; . .env; set +a`) 
1) `cargo migrate up`
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
