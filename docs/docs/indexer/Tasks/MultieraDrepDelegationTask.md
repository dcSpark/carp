# MultieraDrepDelegationTask
Tracks stake delegation actions to dreps


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

   * [MultieraStakeCredentialTask](./MultieraStakeCredentialTask)


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_stake_credential `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_drep_delegation.rs)
