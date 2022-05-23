# MultieraTxCredentialRelationTask
Adds the relation between transactions and credentials that appear within the tx to the database


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct EmptyConfig {}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraAddressTask](./MultieraAddressTask)
   * [MultieraStakeCredentialTask](./MultieraStakeCredentialTask)


## Data accessed
#### Reads from

   * ` multiera_stake_credential `
   * ` vkey_relation_map `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_tx_credential_relations.rs)
