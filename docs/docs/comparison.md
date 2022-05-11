---
sidebar_position: 2
---

# Carp vs alternatives

## Carp vs db-sync

cardano-db-sync is a Haskell application that syncs blockchain data to Postgres. Architecture-wise, db-sync and Carp differ in target audience on two key points:

1. Modularity: Carp uses fully customizable execution plans so that you only index what matters to you. db-sync instead aims for a *batteries included* experience where all structures in Cardano are indexed in case you need them. db-sync does have a plugin system, so both Carp and db-sync allow you to provide more than what is provided by the base experience.
2. Flexibility: db-sync stores data in structures SQL tables so that it can easily be parsed from applications that don't have access to a Cardano SDKs capable of parsing Cardano binary blobs. Carp, on the other hand, assumes you have such a library. An important distinction is that this means db-sync has to be somewhat opinionated about how data is formatted and which data is exposed, whereas Carp exposes the raw binary so that you can access whatever data you need.

Additionally,

- db-sync does not provide a REST api. It does have a graphql API (cardano-graphql), but there is a performance cost to using it.
- db-sync stores ledger state (ex: reward history) to the database whereas Carp only stores information that appears on-chain

## Carp vs Oura

Oura is a Rust tool fetching data from the node, processing it according to rules and filters, and then passing it along to a sink which handles the data. More specifically, Oura itself does not store data anywhere. It simply makes data available to you to decide how to process it.

Carp, on the other hand, takes block cbor, processes it and saves it to Postgres. It can use Oura as a source of block cbor, or it could use some other tool for accessing block data such as a Redis instance.

## Carp vs Kupo

Kupo is a Haskell tool for fetching data from the node and processing it according to rules in filters. The difference is similar to the difference between Carp & Oura

## Carp vs Ogmios

Ogmios is a Haskell tool with a Typescript API to query the cardano-node state such as the current utxo set or the current reward balance. The focus of the project is more towards providing a friendly API to the functionality of the cardano-node, compared to Carp which can be used to efficiently store and process historic information

One main difference is that Carp does not have access to Cardano ledger state that does not explicitly appear on-chain such as reward balances. This means that Ogmios can be a good companion to Carp

## Carp vs Blockfrost

Blockfrost is an api for accessing Cardano data primarily used if you don't want to synchronize your own node. In other words, Blockfrost is an API to an indexer, not an indexer itself like Carp. Blockfrost could choose to make some endpoints from their API powered by Carp.

