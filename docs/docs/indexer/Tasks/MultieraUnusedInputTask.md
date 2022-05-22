# MultieraUnusedInputTask
Adds the unused inputs to the database \(collateral inputs if tx succeeds, collateral inputs otherwise


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

   * [MultieraOutputTask](./MultieraOutputTask)


## Data accessed
#### Reads from

   * ` multiera_txs `


#### Writes to

   * ` vkey_relation_map `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_unused_input.rs)
