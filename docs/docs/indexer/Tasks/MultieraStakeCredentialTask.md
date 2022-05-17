# MultieraStakeCredentialTask
Adds the stake credentials to the database\.
       Note: \`stake credentials\` are an unfortunately poorly named type in the Cardano binary specification\.
       A stake credential has nothing to do with staking\. It's just a hash with an prefix to specify what kind of hash it is \(ex: payment vs script\)

## Era
` multiera `

## Dependencies

   * [MultieraUsedInputTask](./MultieraUsedInputTask)
   * [MultieraUnusedInputTask](./MultieraUnusedInputTask)


## Data accessed
#### Reads from

   * ` multiera_txs `


#### Writes to

   * ` vkey_relation_map `
   * ` multiera_stake_credential `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_stake_credentials.rs)
