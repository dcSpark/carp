# ByronAddressTask
Adds the address raw bytes to the database


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

   * [ByronTransactionTask](./ByronTransactionTask)


## Data accessed
#### Reads from

   * ` byron_txs `


#### Writes to

   * ` byron_addresses `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_address.rs)
