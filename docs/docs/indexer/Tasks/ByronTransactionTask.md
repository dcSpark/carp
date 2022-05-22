# ByronTransactionTask
Adds the transactions in the block to the database


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
` byron `

## Dependencies

   * [ByronBlockTask](./ByronBlockTask)


## Data accessed
#### Reads from

   * ` byron_block `


#### Writes to

   * ` byron_txs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_txs.rs)
