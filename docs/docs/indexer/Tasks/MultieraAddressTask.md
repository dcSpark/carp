# MultieraAddressTask
Adds the address raw bytes to the database

## Era
` multiera `

## Dependencies

   * [MultieraTransactionTask](./MultieraTransactionTask)


## Data accessed
#### Reads from

   * ` multiera_txs `


#### Writes to

   * ` vkey_relation_map `
   * ` multiera_addresses `
   * ` multiera_queued_addresses_relations `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_address.rs)
