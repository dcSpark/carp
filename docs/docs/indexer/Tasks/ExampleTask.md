# ExampleTask
An example task to help people learn how to write custom Carp tasks


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct EmptyConfig {}

```
</details>


## Era
` multiera `

## Data accessed
#### Reads from

   * ` multiera_txs `


#### Writes to

   * ` multiera_addresses `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/dsl/example_task.rs)
