---
sidebar_position: 4
---

# Migrations & Rollbacks

There are times when you may want to resync the database. For example,

1. You are modifying your execution plan
1. You are updating Carp to a new version that includes a breaking change

# Destructive migrations

Utility to easily rollback the database state

Note: this is a destructive action as it will drop all blocks until a given point. This means you will have to resynchronize the database.

- `cargo rollback era 3`
- `cargo rollback epoch 200`
- `cargo rollback height 1000`

Note: these ranges are inclusive. Ex: `epoch 200` means you rollback TO era 200, discarding any epoch afterwards.

# Non-destructive migrations

Given that resync of Carp takes a while, you may want to migrate your database in a non-destructive way. Here are the steps you'll need to follow to do this:

1. If you're making a change to the database schema, create a new database migration in the `indexer/migration` project. You can run migrations with the commands below:

Reminder: you can add the `-v` parameter to see the SQL queries run by these commands

- `cargo migrate up`
- `cargo migrate down`
- `cargo migrate help`

Keep in mind that for successful migration you need to run `set -a; . ./.env; set +a` from root folder of repo to set appropriate env variables (migration relies on them, on `DATABASE_URL` in particular).

2. Create a new execution plan using `readonly = true` versions of tasks. Tasks that support this option will read existing data from storage instead of writing to the database, so you can chain multiple readonly tasks to build up towards that new task you are adding that will write the data you need to the database.
3. Set the `start_block` parameter in your configuration file to the block you want to start synchronizing from (see [here](./run.md))