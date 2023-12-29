---
sidebar_position: 7
---

# Starting Carp in container

Carp can be run using `docker-compose`. It creates 4 (+1 optional) services:

- postgres - container running postgres database
- cardano-node - container running cardano-node
- carp - container build locally with carp
- carp_web - webserver for carp

To run docker deployment you will need to configure .env file, carp's config. We've prepared the necessary configs to run the whole deployment for mainnet. You can run with other networks using the instructions below. 

Run the deployment with the following command from `deployment` folder:

```shell
docker compose --env-file config/indexer/docker.env up -d
```

Setup assumes ports 3000, 3001 and 5432 are available.

## Backup service

There is an optional container that can perform back of the database to a s3 bucket.
It checks every hour the cardano-node tip epoch and if it changes, an SQL backup is being generated.
By default, the backup service is commented out. At the moment only mainnet and testnet are supported by backuper.

Backup service will upload file to path:
`s3://${S3_BUCKET}/${S3_FOLDER}/${NETWORK}/`

**WARNING:**
Starting Carp with backup service from scratch will create a dozens of backups until cardano-node fully synchronizes.

## Environment setup

Some environment variables must be set up to ensure successful running of all containers. We've prepared an example .env file: [docker.env](https://github.com/dcSpark/carp/blob/main/deployment/config/indexer/docker.env). It contains mandatory variables already:
```dotenv
# network can be mainnet/preview/preprod/testnet
# this parameter is utilized by migration service and docker-compose
NETWORK=mainnet

# can be cardano_node_docker.yml or oura_docker.yml or 
# any custom filename located in deployment/config/indexer folder 
# alternative to CARP_CONFIG env variable
CONFIG_FILE=cardano_node_docker.yml

CARP_VERSION=2.4.0

# these credentials are utilized by postgres, carp and carp_web services
# host should be container name or static ip
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_DB=carp_mainnet
PGUSER=carp
PGPASSWORD=1234
PGPASSFILE="$(realpath secrets/.pgpass)"

# note: PGPASSWORD isn't required to run carp
# since it will be parsed from the PGPASSFILE instead
# as this command will gracefully fallback to PGPASSFILE if no password is specified
# However, some dev tools like pgtyped & zapatos don't support .pgpass files
DATABASE_URL=postgresql://${PGUSER}:${PGPASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}
# Needed for PgTyped
PGURI=$DATABASE_URL
```

Please set `NETWORK`, `CONFIG_FILE` and postgres variables carefully. `CONFIG_FILE` should be located in `deployment/config/indexer` folder. You can use `CARP_CONFIG` env variable instead of `CONFIG_FILE` if you want to use just env variables for configuration.

You can also add non-mandatory variables that are useful for backuper:

```bash
# optional content for backup service
S3_BUCKET=<name of bucket to store backups>
S3_FOLDER=<name of folder inside s3 bucket. I recommend host name>
AWS_ACCESS_KEY_ID=<Access key for given s3 bucket with creation permissions>
AWS_SECRET_ACCESS_KEY=<Secret for given account>
```

## `.pgpass` location

`.pgpass` file should be stored in `deployment/config/secrets/<NETWORK>/.pgpass`.

## Carp configuration

We mentioned config files above, they should be stored in `deployment/config/indexer` folder, and you should set `CONFIG_FILE` env variable with the config name. Alternatively you can set `CARP_CONFIG` env variable and not use the configuration file

The difference from non-docker configuration is in the networking mainly.

### Sink configuration changes

For sink you should set host to postgres container name from docker-compose: `postgres`. Other logic is the same. This change is mandatory, since docker network resolution uses container names (ips are non-static in general case and localhost won't work either)

### Source configuration changes

#### Oura

For oura source you can use `Unix` socket:

```yaml
source:
  type: oura
  socket: "/app/node-ipc/node.socket"
  bearer: Unix
```

Alternatively you can use `Tcp` as well:

```yaml
source:
  type: oura
  socket: "172.20.0.4:3001"
  bearer: Tcp
```

#### Cardano_net

For `cardano_net` you can use `Tcp` only:

```yaml
source:
  type: cardano_net
  relay:
    - 172.20.0.4
    - 3001
```

**Note**: `cardano-node` container has static ip `172.20.0.4` in docker-compose.

**WARNING**: `cardano_net` can't resolve docker names itself, that's why static ip is assigned.

## Troubleshooting

Docker deployment shares with local filesystem in paths:
```shell
# cardano-node container
deployment/<NETWORK>/node-ipc
deployment/<NETWORK>/node-db

# postgres container
deployment/<NETWORK>/postgres-data

# carp container
deployment/<NETWORK>/node-ipc # to read unix socket
deployment/config/indexer/ # to get config for carp
```

If at any point you broke the consistency (by running multiple deployments with access to the same folders / running incompatible software versions), you can remove non-config folders from above (but all progress will be **lost**).

Don't hesitate to submit issues to [carp repository](https://github.com/dcSpark/carp/issues) as well.
