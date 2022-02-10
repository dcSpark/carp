# Oura Postgres Sink

**status**: in development

Sync a postgres database with the cardano blockchain using [Oura](https://github.com/txpipe/oura)

## Usage

### Running

`cargo run`

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
