# MultieraAddressCredentialRelationTask
Adds to the database the relation between addresses and the credentials part of the addresses \(ex: payment key \+ staking key\)


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct ReadonlyConfig {
    pub readonly: bool,
}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraAddressTask](./MultieraAddressTask)
   * [MultieraStakeCredentialTask](./MultieraStakeCredentialTask)


## Data accessed
#### Reads from

   * ` multiera_addresses `
   * ` multiera_queued_addresses_relations `
   * ` multiera_stake_credential `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_address_credential_relations.rs)
