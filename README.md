# Carp (Cardano Postgres Indexer)

# Running

This project contains two different parts to it:

1. An in [indexer README](./indexer/README.md) which stores the chain to a Postgres database
2. A [webserver README]('./webserver/README.md) which provides a REST API for querying the database (ex: for light wallets)

You may also be interested in:

1. The webserver [swagger / OpenAPI](https://dcspark.github.io/carp/#/)
2. The [generated SQL](./webserver/server/bin/schema.sql) if you want to run your own queries
3. A [visualization](./webserver/server/bin/graph.png) of the generated SQL
