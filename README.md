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
1) `sudo -u postgres psql -c "\password oura"`
1) Add your database name & user password to `secrets/.pgpass`
1) Modify the env variables in `.env` if needed (ex: connecting to local node instead of remote)
1) Run the env file (`set -a; . .env; set +a`) - note you will have to re-run this command every time you reopen your shell
1) Run `export HBA_PATH=$(sudo -u postgres psql -c "show hba_file;" | sed -n '3p' | xargs echo -n)`
1) Run `sudo -E sh -c 'echo "local $POSTGRES_DB $PGUSER md5" >> $HBA_PATH'`
1) Run `sudo -u postgres psql -c "SELECT pg_reload_conf();"`
1) `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)
1) `cargo run`

psql -h localhost -U oura -d oura_postgres_mainnet -c "SELECT 5"
psql -U oura -d oura_postgres_mainnet -c "SELECT 5"
psql "postgresql://localhost:5432/oura_postgres_mainnet" -c "SELECT 5"

### Migrations

There is an alias configured for convenience.

- `cargo migrate up`
- `cargo migrate down`
- `cargo migrate help`

### Web Server

Once you've built and run the project, you can of course run any SQL query you want.

However, if you want some queries useful for light wallets and similar applications, you can check out the `webserver` folder.

## Crates

- **entity**
  - contains the SeaORM models
- **migration**
  - contains the database migrations
- **src**
  - contains the application code
