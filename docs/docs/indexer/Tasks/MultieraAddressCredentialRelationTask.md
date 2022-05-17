# MultieraAddressCredentialRelationTask
Adds to the database the relation between addresses and the credentials part of the addresses \(ex: payment key \+ staking key\)

## Era
` multiera `

## Dependencies

   * [MultieraAddressTask](./MultieraAddressTask)
   * [MultieraStakeCredentialTask](./MultieraStakeCredentialTask)


## Data accessed
#### Reads from

   * ` multiera_queued_addresses_relations `
   * ` multiera_stake_credential `


#### Writes to

   * ` multiera_addresses `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_address_credential_relations.rs)
