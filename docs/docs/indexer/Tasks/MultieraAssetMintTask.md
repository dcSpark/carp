# MultieraAssetMintTask
Adds new tokens and keeps track of mints/burns in general


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

   * ` multiera_block `
   * ` multiera_txs `


#### Writes to

   * ` multiera_assets `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_asset_mint.rs)
