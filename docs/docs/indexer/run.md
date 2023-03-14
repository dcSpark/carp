---
sidebar_position: 3
---

# Running

To run carp you need to configure carp itself, set up / configure cardano-node (or use remote one), configure postgres and env variables. For mainnet we provide example [.env file](https://github.com/dcSpark/carp/blob/main/.env) and [carp default configuration](https://github.com/dcSpark/carp/blob/main/indexer/configs/default.yml), so you can jump directly to `setting up cardano-node section`.

## Configuration & concepts

Carp itself uses special file for configuration, the examples are stored [there](https://github.com/dcSpark/carp/blob/main/indexer/configs/). Besides, carp's config can be set up through `CARP_CONFIG` env variable in json format. Let's dive into configuration a little further:

File format:
```yaml
source:
  type: oura
  socket: "relays-new.cardano-mainnet.iohk.io:3001"
  bearer: Tcp # Unix

sink:
  type: cardano
  db:
    type: postgres
    database_url: postgresql://carp:1234@localhost:5432/carp_mainnet
  network: mainnet # preview / preprod / testnet

start_block:
```

Json format:
```json
{"source":{"type":"oura","socket":"relays-new.cardano-mainnet.iohk.io:3001","bearer":"Tcp"},"sink":{"type":"cardano","db":{"type": "postgres","database_url":"postgresql://carp:1234@localhost:5432/carp_mainnet"},"network":"mainnet"},"start_block":null}
```

As you might see there are several key sections: source and sink. For sink there's only one option at the moment: `cardano` sink. For source there are two options with different configurations: `oura` and `cardano_net`.

### Sink configuration

Cardano sink configuration requires the `type`, `db` and `network` to be configured. 

Supported values for `type`:
* `cardano`

Supported values for `network`:
* `mainnet`
* `preprod`
* `preview`
* `testnet`

Supported values for `db`:
* `postgres`

In `db` settings mind the host: in case of docker deployment `localhost` won't work, you will need to set static ip or container name there.

### Source configuration

There are two types of sources: `oura` and `cardano_net`.

#### Oura source parameters
```yaml
source:
  type: oura
  socket: "relays-new.cardano-mainnet.iohk.io:3001"
  bearer: Tcp # Unix
```

If you work with remote node and plan to fetch data through `Tcp`:
```yaml
source:
  type: oura
  socket: "remote.node.url"
  bearer: Tcp # Unix
```

If you work with local node you can fetch data through `Unix` socket as well:
```yaml
source:
  type: oura
  socket: "path.to.socket"
  bearer: Unix # Tcp
```

#### Cardano_net source parameters
```yaml
source:
  type: cardano_net
  relay: 
    - relays-new.cardano-mainnet.iohk.io
    - 3001
      # - preview-node.world.dev.cardano.org
      # - 30002
      # - preprod-node.world.dev.cardano.org
      # - 30000
```

To use `cardano_net` source you should set up `relay` and provide url and port. `Unix` socket is not supported here.

## Setting up cardano-node

The indexer can work with either local or remote node. 

### Local node

For local node you will have to run a synced copy of [cardano-node](https://github.com/input-output-hk/cardano-node/) and update the configuration file accordingly:
* for `cardano_net`:
  * Expose port `3001` (default port in cardano node)
  * Update the `relay` with appropriate ip and port 
* for `oura`
  * Choose whether you want to use unix socket or tcp
  * Update `bearer` and `socket` fields in the config accordingly (more details above).

### Remote node

For remote node you will need to update carp's config:
* for `cardano_net`:
    * Update the relay with appropriate ip and port
* for `oura`
    * Update `bearer` to `Tcp`
    * Update `socket` to the relay address

## Setting up the database

Note: steps assume mainnet

1. `sudo -u postgres createdb carp_mainnet`
2. `sudo -u postgres psql -c 'ALTER DATABASE carp_mainnet SET jit_above_cost = -1;'`
3. `sudo -u postgres createuser carp`
4. `sudo -u postgres psql -c "\password carp"`
5. For postgres version 15+ there's a breaking change regarding the permissions, so you will need to run extra command:
   1. `sudo -u postgres psql -t -d carp_mainnet -c 'GRANT ALL ON SCHEMA public TO carp;'`
6. Add your database name, username and password to `secrets/.pgpass`
7. `chmod 600 secrets/.pgpass`
8. Modify the env variables in `.env` if needed:
   1. Setting `POSTGRES_HOST`, `POSTGRES_PORT`, `PGUSER`, `PGPASSWORD`, `POSTGRES_DB`, `PGPASSFILE` variables is mandatory to run migrations successfully.
9. Modify the carp's config variables for sink:
   1. `host`, `port`, `user`, `password` and `db` must be set the same values as you created with steps 1-4 and set at step 8
10. In the root project folder, run `set -a; . ./.env; set +a` - note you will have to re-run this command every time you reopen your shell
    1. This step is mandatory to run the migration, it still relies on env variables.
11. Inside the `indexer` folder, run `cargo migrate up` (you can debug migration by adding a `-v` at the end of the command)
    1. This creates the necessary tables

## Running the indexer

Inside the `indexer` folder, run `cargo run -- --plan execution_plans/default.toml --config-path configs/oura.yml`

Reminder: you can visualize the execution plan using `cargo plan-visualizer --plan execution_plans/default.toml -o plan-visualizer/out`

You can use other configs and write your own: e.g. `configs/cardano_node.yml` in the `indexer` folder.

## Detailed environment setup

For non-docker deployment the following variables are mandatory to be configured:

```dotenv
# network can be mainnet/preview/preprod/testnet
# this parameter is utilized by migration service, cardano-node, backuper
NETWORK=mainnet

# these credentials are utilized by postgres, carp and carp_web services
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
PGUSER=carp
PGPASSWORD=""
POSTGRES_DB=carp_mainnet
PGPASSFILE="$(realpath secrets/.pgpass)"

# note: PGPASSWORD isn't required to run carp
# since it will be parsed from the PGPASSFILE instead
# as this command will gracefully fallback to PGPASSFILE if no password is specified
# However, some dev tools like pgtyped & zapatos don't support .pgpass files
DATABASE_URL=postgresql://${PGUSER}:${PGPASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}
# Needed for PgTyped
PGURI=$DATABASE_URL
```

Variables related to postgres are described above in `Setting up the database` section.
