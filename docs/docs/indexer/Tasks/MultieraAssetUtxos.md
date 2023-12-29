# MultieraAssetUtxos
Parses utxo movements for native assets


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

   * [MultieraUsedInputTask](./MultieraUsedInputTask)
   * [MultieraOutputTask](./MultieraOutputTask)


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_outputs `
   * ` multiera_used_inputs_to_outputs_map `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/indexer/tasks/src/multiera/multiera_asset_utxo.rs)
