---
sidebar_position: 7
---

# Starting Carp in container

Carp can be run using `docker-compose`. It creates 3(+1 optional) services:

- postgres - container running postgres database
- cardano-node - container running cardano-node
- carp - container build locally with carp

### Backup service

There is an optional container that can perform back of the database to an s3 bucket.
It check every hour the cardano-node tip epoch and if it changes, an SQL backup is being generated.
By defaul the backup service is commented out.

Backup service will upload file to path:
`s3://${S3_BUCKET}/${S3_FOLDER}/${NETWORK}/`

**WARNING:**
Starting Carp with backup service from scratch will create a dozens of backups until cardano-node fully synchronizes.

### Environment setup

For Your convenience You can create a `.env` file alongside `docker-compose.yml` file with:

```bash
NETWORK=<network type: mainnet/testnet/preview/preprod>

POSTGRES_USER=<user used for carp logging into DB>
POSTGRES_PASSWORD=<carp DB name>
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_DB=<password for carp user>

DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}
# optional content for backup service
S3_BUCKET=<name of bucket to store backups>
S3_FOLDER=<name of folder inside s3 bucket. I recommend host name>
AWS_ACCESS_KEY_ID=<Access key for given s3 bucket with creation permissions>
AWS_SECRET_ACCESS_KEY=<Secret for given account>
```
