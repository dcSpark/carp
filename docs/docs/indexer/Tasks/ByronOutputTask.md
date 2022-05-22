# ByronOutputTask
Adds the transaction outputs to the database


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct EmptyConfig {}

```
</details>


## Era
` byron `

## Dependencies

   * [ByronAddressTask](./ByronAddressTask)


## Data accessed
#### Reads from

   * ` byron_txs `
   * ` byron_addresses `


#### Writes to

   * ` byron_outputs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_outputs.rs)
