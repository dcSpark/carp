# MultieraTransactionTask
Adds the transactions in the block to the database


<details>
    <summary>Configuration</summary>

```rust
use super::PayloadConfig::PayloadConfig;
use super::ReadonlyConfig::ReadonlyConfig;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct PayloadAndReadonlyConfig {
    pub include_payload: bool,
    pub readonly: bool,
}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraBlockTask](./MultieraBlockTask)


## Data accessed
#### Reads from

   * ` multiera_block `


#### Writes to

   * ` multiera_txs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_txs.rs)
