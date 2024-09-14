# Setup

1. Follow the DB setup steps in the [root README](../../README.md)
1. Note: unlike the Rust tool, the Typescript codebase doesn't support `PGPASSFILE` so you will have to set the `PGPASSWORD` env variable (not set by .env)
1. `nvm use`
1. `yarn install`

<!-- set -a; pushd ../../ && . ./.env; popd; set +a -->

# Run

1. (dev) `yarn dev:start`
1. (prod) `yarn build && yarn prod:start`

# Regenerate database

Requirements:

- You will need to run `pgtyped` for this (which requires `PGPASSWORD`)
- You will need to have pipenv setup (see [gen-graph.sh](./bin/gen-graph.sh) for setup instructions)

`yarn parse-db`
