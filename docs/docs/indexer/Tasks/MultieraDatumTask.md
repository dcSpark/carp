# MultieraDatumTask
Adds datum and datum hashes


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

   * [MultieraTransactionTask](./MultieraTransactionTask)


## Data accessed
#### Reads from

   * ` multiera_txs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_datum.rs)
