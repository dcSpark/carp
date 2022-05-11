---
sidebar_position: 4
---

# Resyncing database

There are times when you may want to resync the database. For example,

1. You are modifying your execution plan
1. You are updating Carp to a new version that includes a breaking change

There are two utilities that will help with this

### Migration utility

There is an alias configured for convenience.

Reminder: you can add the `-v` parameter to see the SQL queries run by these commands

- `cargo migrate up`
- `cargo migrate down`
- `cargo migrate help`

### Rollback utility

Utility to easily rollback the database state

- `cargo rollback era 3`
- `cargo rollback epoch 200`
- `cargo rollback height 1000`

Note: these ranges are inclusive. Ex: `epoch 200` means you rollback TO era 200, discarding any epoch afterwards.
