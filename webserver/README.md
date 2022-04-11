# Setup

1. Follow the DB setup steps in the [root README](../README.md)
1. If you need to run the dev tools like Prisma, you also need to set the `PGPASSWORD` env variable (not set by .env) as Prisma doesn't support PGPASSFILE
1. `nvm use`
1. `yarn install`

<!-- set -a; pushd ../ && . ./.env; popd; set +a -->

# Run

1. `yarn start`

# Regenerate database

You will need to run Prisma for this (which requires `PGPASSWORD`)

`npm run db-dump`
