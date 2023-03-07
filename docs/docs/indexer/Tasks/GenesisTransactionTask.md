# GenesisTransactionTask
Parses Genesis transactions \(avvm & non\-avvm balances from genesis\)


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct PayloadConfig {
    pub include_payload: bool,
}

```
</details>


## Era
` genesis `

## Dependencies

   * [GenesisBlockTask](./GenesisBlockTask)


## Data accessed
#### Reads from

   * ` genesis_block `


#### Writes to

   * ` genesis_txs `
   * ` genesis_addresses `
   * ` genesis_outputs `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/genesis/genesis_txs.rs)
