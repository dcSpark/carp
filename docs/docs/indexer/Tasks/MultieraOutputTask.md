# MultieraOutputTask
Adds the used outputs to the database \(regular inputs in most cases, collateral inputs if tx fails\)


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


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_addresses `


#### Writes to

   * ` multiera_outputs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_used_outputs.rs)
