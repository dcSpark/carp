# Carp - Test it!


#### Run environment and create backup
- Edit `.env` file to justify to your needs
```
INSTANCE - name of instance - useful when multiple instances will be run simultaneously
COMPOSE_PROJECT_NAME - name of instance - useful when multiple instances will be run simultaneously
CARDANO_PORT=3001 - port on which cardano will be visible in OS level
CARP_WEB_PORT=3000 - port on which carp will be visible in OS level 

NETWORK - cardano network (mainnet/preprod/preview/testnet)
TOP_DIR - where all the networks will be stored - running user has to have access to write files there 
POSTGRES_PASSWORD - carp db password
POSTGRES_DB - carp db
POSTGRES_USER - carp user
POSTGRES_HOST - carp host
DATABASE_URL - carp database url ( combination of above postgresql://carp:example@carp-postgres:5432/carp )
SNAPSHOTS_TOP_DIR=/opt/snapshots 
UID - uid of running user (important to set to be able to do backups)
GID - gid of running user (important to set to be able to do backups)
```


- Run docker-compose
```
docker-compose up -d
```

- Create snapshot and backup - this needs to be run as ***sudo***
```
sudo ./create_snapshot.sh
```

After this in local directory tar with backup of cardano and carp database will be created.

#### Restore 

- Copy created tar file to another directory
- Untar tar file
- Change TOP_DIR in .env settings
- `docker-compose up` and enjoy!

