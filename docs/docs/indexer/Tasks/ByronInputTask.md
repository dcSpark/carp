# ByronInputTask
Adds the transaction inputs to the database


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

   * [ByronOutputTask](./ByronOutputTask)


## Data accessed
#### Reads from

   * ` byron_txs `


#### Writes to

   * ` byron_inputs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_inputs.rs)
