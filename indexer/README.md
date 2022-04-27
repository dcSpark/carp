# oura-postgres-sink Indexer

`cargo run` will start oura-postgres-sink.

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
